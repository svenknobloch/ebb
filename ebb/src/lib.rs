#![feature(type_alias_impl_trait)]
// Required to use proc-macro in crate
extern crate self as ebb;

mod channels;
pub mod components;
mod network;
mod ports;
mod process;

pub use channels::*;
pub use network::*;
pub use ports::*;
pub use process::*;

pub use ebb_macros::Ports;

pub use thread_local::*;

mod thread_local {
    use std::cell::RefCell;
    use std::future::Future;
    use std::sync::Arc;

    use smol::Task;

    use crate::{Network, Ports, Process};

    std::thread_local!(pub static NETWORK: RefCell<Network> = Default::default());

    pub fn spawn_local_process<P>(process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: 'static,
    {
        NETWORK.with(|network| network.borrow().spawn_local_process(process))
    }

    pub fn spawn_process<P>(process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: Send + 'static,
    {
        NETWORK.with(|network| network.borrow().spawn_process(process))
    }

    pub fn spawn_local_task<F, T: 'static>(task: F) -> Task<T>
    where
        F: Future<Output = T> + 'static,
    {
        NETWORK.with(|network| network.borrow().spawn_local_task(task))
    }

    pub fn spawn_task<F, T: Send + 'static>(task: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        NETWORK.with(|network| network.borrow().spawn_task(task))
    }

    pub fn tick() -> bool {
        NETWORK.with(|network| network.borrow().tick())
    }

    pub fn run<T>(f: impl Future<Output = T>) -> T {
        NETWORK.with(|network| network.borrow().run(f))
    }

    pub fn complete() {
        NETWORK.with(|network| network.borrow().complete())
    }
}
