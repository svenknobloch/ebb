use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::Stream;

use crate::{Channel, NetworkConfig, Ports};

#[derive(Debug)]
pub struct ReceiverHandle<T> {
    channel: Arc<Channel<T>>,
}

impl<T> ReceiverHandle<T> {
    pub(crate) fn channel(&self) -> &Arc<Channel<T>> {
        &self.channel
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(item) = self.channel.pop() {
            self.channel.wake_tx();
            Poll::Ready(Some(item))
        } else {
            self.channel.register_rx_waker(cx.waker());

            if let Some(item) = self.channel.pop() {
                self.channel.wake_tx();
                Poll::Ready(Some(item))
            } else {
                Poll::Pending
            }
        }
    }
}

impl<T: 'static> Ports for Receiver<T> {
    type Handle = ReceiverHandle<T>;

    fn handle(&self) -> Self::Handle {
        Self::Handle { channel: self.channel.clone() }
    }

    fn create(config: &NetworkConfig) -> Self {
        let channel = Arc::new(Channel::with_capacity(config.buffer_size));

        Self {
            channel: channel.clone(),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.channel.close();
    }
}
