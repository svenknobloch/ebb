use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver as StdReceiver, Sender as StdSender};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Sink;

use crate::{Channel, NetworkConfig, Ports, ReceiverHandle, SendError};

#[derive(Debug)]
pub struct BroadcastHandle<T: Clone> {
    ctrl: StdSender<Arc<Channel<T>>>,
}

impl<T: Clone> BroadcastHandle<T> {
    pub fn connect(&self, rx: &ReceiverHandle<T>) {
        self.ctrl.send(rx.channel().clone()).ok();
    }
}

// TODO: fix issues on sending to full channel

#[derive(Debug)]
pub struct Broadcast<T: Clone> {
    ctrl_tx: StdSender<Arc<Channel<T>>>,
    ctrl_rx: StdReceiver<Arc<Channel<T>>>,
    channels: Vec<Arc<Channel<T>>>,
}

impl<T: Clone> Default for Broadcast<T> {
    fn default() -> Self {
        let (ctrl_tx, ctrl_rx) = channel();

        Self {
            ctrl_tx,
            ctrl_rx,
            channels: Default::default(),
        }
    }
}

impl<T: Clone> Sink<T> for Broadcast<T> {
    type Error = SendError<T>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = &mut *self;

        // Update channels
        inner.channels.extend(inner.ctrl_rx.try_iter());

        // Poll ready
        for channel in &mut self.channels {
            if channel.is_full() {
                channel.register_tx_waker(cx.waker());
                return Poll::Pending;
            }
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        // TODO: Check closed channels

        for channel in &self.channels {
            channel.push(item.clone())?;
        }

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for channel in &self.channels {
            channel.wake_rx();
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for channel in &self.channels {
            channel.wake_rx();
        }

        Poll::Ready(Ok(()))
    }
}

impl<T: Clone + 'static> Ports for Broadcast<T> {
    type Handle = BroadcastHandle<T>;

    fn handle(&self) -> Self::Handle {
        Self::Handle { ctrl: self.ctrl_tx.clone() }
    }

    fn create(_: &NetworkConfig) -> Self {
        Default::default()
    }
}
