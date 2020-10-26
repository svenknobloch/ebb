use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver as StdReceiver, Sender as StdSender};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Sink;

use crate::{Channel, Network, Ports, ReceiverHandle, SendError};

#[derive(Debug)]
pub struct SenderHandle<T> {
    ctrl: StdSender<Arc<Channel<T>>>,
}

impl<T> SenderHandle<T> {
    pub fn connect(&self, rx: &ReceiverHandle<T>) {
        self.ctrl.send(rx.channel().clone()).ok();
    }
}

#[derive(Debug)]
pub struct Sender<T> {
    ctrl: StdReceiver<Arc<Channel<T>>>,
    tx: Option<Arc<Channel<T>>>,
}

impl<T> Sender<T> {
    pub fn connect(&mut self, rx: &ReceiverHandle<T>) {
        self.tx = Some(rx.channel().clone());
    }
}

impl<T> Default for Sender<T> {
    fn default() -> Self {
        let (_, ctrl) = channel();
        Self { ctrl, tx: None }
    }
}

impl<T> Sink<T> for Sender<T> {
    type Error = SendError<T>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = &mut *self;

        // Update sender
        if let Some(tx) = inner.ctrl.try_iter().last() {
            inner.tx = Some(tx);
        }

        // Check sender ready
        if let Some(channel) = self.tx.as_ref() {
            if channel.is_full() {
                channel.register_tx_waker(cx.waker());
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        if let Some(channel) = self.tx.as_ref() {
            channel.push(item)
        } else {
            // TODO: Warn about silently dropping values
            // println!("Dropping Value");
            Ok(())
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(channel) = self.tx.as_ref() {
            channel.wake_rx();
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(channel) = self.tx.as_ref() {
            channel.wake_rx();
        }

        Poll::Ready(Ok(()))
    }
}

impl<T: 'static> Ports for Sender<T> {
    type Handle = SenderHandle<T>;

    fn with_handle(_: &Network) -> (Self, Self::Handle) {
        let (ctrl_tx, ctrl_rx) = channel();

        (
            Self {
                ctrl: ctrl_rx,
                tx: None,
            },
            Self::Handle { ctrl: ctrl_tx },
        )
    }
}
