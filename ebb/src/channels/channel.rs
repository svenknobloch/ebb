use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::task::Waker;

use crossbeam_queue::ArrayQueue;
use futures::task::AtomicWaker;

#[derive(Copy, Clone)]
pub enum SendError<T> {
    Closed,
    Full(T),
}

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Closed => f.debug_tuple("Closed").finish(),
            SendError::Full(_) => f.debug_tuple("Full").finish(),
        }
    }
}

#[derive(Debug)]
pub struct Channel<T> {
    closed: AtomicBool,
    tx_wakers: Mutex<Vec<Waker>>,
    rx_waker: AtomicWaker,
    queue: ArrayQueue<T>,
}

impl<T> Channel<T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            closed: Default::default(),
            tx_wakers: Default::default(),
            rx_waker: Default::default(),
            queue: ArrayQueue::new(capacity),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    pub fn register_tx_waker(&self, waker: &Waker) {
        self.tx_wakers.lock().unwrap().push(waker.clone());
    }

    pub fn register_rx_waker(&self, waker: &Waker) {
        self.rx_waker.register(waker);
    }

    pub fn wake_rx(&self) {
        self.rx_waker.wake();
    }

    pub fn wake_tx(&self) {
        if let Some(waker) = self.tx_wakers.lock().unwrap().pop() {
            waker.wake()
        }
    }

    pub fn push(&self, item: T) -> Result<(), SendError<T>> {
        self.queue.push(item).map_err(|e| SendError::Full(e))
    }

    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }
}
