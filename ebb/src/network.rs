use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use smol::channel::{unbounded, Receiver, Sender};
use smol::{Executor, LocalExecutor, Task};

use crate::{Ports, Process, NETWORK};

#[derive(Copy, Clone)]
pub struct NetworkConfig {
    pub buffer_size: usize,
    pub num_threads: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64,
            num_threads: num_cpus::get(),
        }
    }
}

#[derive(Default)]
pub struct NetworkBuilder {
    config: NetworkConfig,
}

impl NetworkBuilder {
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.config.buffer_size = buffer_size;
        self
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.config.buffer_size = num_threads;
        self
    }

    pub fn build(self) -> Network {
        let (tx, rx) = unbounded();

        let network = Network {
            config: self.config,
            exec_local: Default::default(),
            exec: Default::default(),
            active: Default::default(),
            shutdown_tx: tx,
            shutdown_rx: rx,
        };

        (0..self.config.num_threads - 1).for_each(|_| {
            let exec = network.exec.clone();
            let tx = network.shutdown_tx.clone();
            let rx = network.shutdown_rx.clone();
            let config = self.config;
            let active = network.active.clone();

            std::thread::spawn(move || {
                Network {
                    config,
                    exec_local: Default::default(),
                    exec,
                    active,
                    shutdown_tx: tx,
                    shutdown_rx: rx,
                }
                .complete()
            });
        });

        network
    }
}

pub struct Network {
    config: NetworkConfig,
    exec_local: LocalExecutor<'static>,
    exec: Arc<Executor<'static>>,
    active: Arc<AtomicUsize>,
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

impl Drop for Network {
    fn drop(&mut self) {
        self.shutdown_tx.close();
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Network {
    pub fn builder() -> NetworkBuilder {
        Default::default()
    }

    pub fn install(self) {
        NETWORK.with(|cell| {
            *cell.borrow_mut() = self;
        });
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn spawn_local_process<P>(&self, process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: 'static,
    {
        let ports = <P::Ports as Ports>::create(self.config());
        let handle = ports.handle();

        let shutdown = self.shutdown_tx.clone();
        let active = self.active.clone();
        let future = process.execute(ports);
        active.fetch_add(1, Ordering::AcqRel);

        self.exec_local
            .spawn(async move {
                future.await;
                if active.fetch_sub(1, Ordering::AcqRel) == 0 {
                    shutdown.close();
                }
            })
            .detach();

        Arc::new(handle)
    }

    pub fn spawn_process<P>(&self, process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: Send + 'static,
    {
        let ports = <P::Ports as Ports>::create(self.config());
        let handle = ports.handle();

        let shutdown = self.shutdown_tx.clone();
        let active = self.active.clone();
        let future = process.execute(ports);
        active.fetch_add(1, Ordering::AcqRel);

        self.exec
            .spawn(async move {
                future.await;
                if active.fetch_sub(1, Ordering::AcqRel) == 0 {
                    shutdown.close();
                }
            })
            .detach();

        Arc::new(handle)
    }

    pub fn spawn_local_task<F, T: 'static>(&self, task: F) -> Task<T>
    where
        F: Future<Output = T> + 'static,
    {
        self.exec_local.spawn(task)
    }

    pub fn spawn_task<F, T: Send + 'static>(&self, task: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        self.exec.spawn(task)
    }

    pub fn tick(&self) -> bool {
        self.exec_local.try_tick() || self.exec.try_tick()
    }

    pub fn run<T>(&self, f: impl Future<Output = T>) -> T {
        smol::block_on(self.exec_local.run(self.exec.run(f)))
    }

    pub fn complete(&self) {
        smol::block_on(self.exec_local.run(self.exec.run(self.shutdown_rx.recv()))).ok();
    }
}
