use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::task::AtomicWaker;
use smol::{Executor, LocalExecutor, Task};

use crate::{Ports, Process};

#[derive(Clone, Default)]
pub struct NetworkStatus {
    waker: Arc<AtomicWaker>,
    process_count: Arc<AtomicUsize>,
}

impl Future for NetworkStatus {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.waker.register(cx.waker());

        if self.process_count.load(Ordering::Acquire) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[derive(Default)]
pub struct Network {
    exec_local: Arc<LocalExecutor<'static>>,
    exec: Arc<Executor<'static>>,
    status: NetworkStatus,
}

impl Network {
    pub fn spawn_local<P>(&mut self, process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: 'static,
    {
        let (ports, handle) = <P::Ports as Ports>::with_handle(&self);

        let status = self.status.clone();
        let future = process.execute(&self, ports);
        status.process_count.fetch_add(1, Ordering::AcqRel);

        self.exec_local
            .spawn(async move {
                future.await;
                status.process_count.fetch_sub(1, Ordering::AcqRel);
                status.waker.wake();
            })
            .detach();

        Arc::new(handle)
    }

    pub fn spawn<P>(&mut self, process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: Send + 'static,
    {
        let (ports, handle) = <P::Ports as Ports>::with_handle(&self);

        let status = self.status.clone();
        let future = process.execute(&self, ports);
        status.process_count.fetch_add(1, Ordering::AcqRel);

        self.exec
            .spawn(async move {
                future.await;
                status.process_count.fetch_sub(1, Ordering::AcqRel);
                status.waker.wake();
            })
            .detach();

        Arc::new(handle)
    }

    pub fn spawn_task_local<F, T: 'static>(&mut self, task: F) -> Task<T>
    where
        F: Future<Output = T> + 'static,
    {
        self.exec_local.spawn(task)
    }

    pub fn spawn_task<F, T: Send + 'static>(&mut self, task: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        self.exec.spawn(task)
    }

    pub fn tick(&self) {
        while self.exec_local.try_tick() || self.exec.try_tick() {}
    }

    pub fn run<T>(&self, f: impl Future<Output = T>) -> T {
        smol::block_on(self.exec_local.run(self.exec.run(f)))
    }

    pub fn execute(self) {
        smol::block_on(self.exec_local.run(self.exec.run(self.status.clone())));
    }
}
