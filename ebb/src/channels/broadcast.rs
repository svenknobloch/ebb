use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver as StdReceiver, Sender as StdSender};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::ops::Shr;

use futures::Sink;

use crate::{Channel, NetworkConfig, Ports, ReceiverHandle, Receiver, SendError};

#[derive(Debug)]
pub struct BroadcasterHandle<T: Clone> {
    ctrl: StdSender<Arc<Channel<T>>>,
}

impl<T: Clone> Shr<&ReceiverHandle<T>> for &BroadcasterHandle<T> {
    type Output = ();
    
    fn shr(self, rx: &ReceiverHandle<T>) -> Self::Output {
        self.ctrl.send(rx.channel().clone()).ok();
    }
}

impl<T: Clone> Shr<&Receiver<T>> for &BroadcasterHandle<T> {
    type Output = ();
    
    fn shr(self, rx: &Receiver<T>) -> Self::Output {
        self.ctrl.send(rx.channel().clone()).ok();
    }
}

// TODO: fix issues on sending to full channel

#[derive(Debug)]
pub struct Broadcaster<T: Clone> {
    ctrl_tx: StdSender<Arc<Channel<T>>>,
    ctrl_rx: StdReceiver<Arc<Channel<T>>>,
    channels: Vec<Arc<Channel<T>>>,
}

impl<T: Clone> Shr<&ReceiverHandle<T>> for &mut Broadcaster<T> {
    type Output = ();
    
    fn shr(self, rx: &ReceiverHandle<T>) -> Self::Output {
        self.channels.push(rx.channel().clone());
    }
}

impl<T: Clone> Shr<&Receiver<T>> for &mut Broadcaster<T> {
    type Output = ();
    
    fn shr(self, rx: &Receiver<T>) -> Self::Output {
        self.channels.push(rx.channel().clone());
    }
}

impl<T: Clone> Default for Broadcaster<T> {
    fn default() -> Self {
        let (ctrl_tx, ctrl_rx) = channel();

        Self {
            ctrl_tx,
            ctrl_rx,
            channels: Default::default(),
        }
    }
}

impl<T: Clone> Sink<T> for Broadcaster<T> {
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

impl<T: Clone + 'static> Ports for Broadcaster<T> {
    type Handle = BroadcasterHandle<T>;

    fn handle(&self) -> Self::Handle {
        Self::Handle { ctrl: self.ctrl_tx.clone() }
    }

    fn create(_: &NetworkConfig) -> Self {
        Default::default()
    }
}
