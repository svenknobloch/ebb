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
    use std::future::Future;
    use std::sync::Arc;

    use smol::Task;

    use crate::{Network, Ports, Process};

    scoped_tls::scoped_thread_local!(pub static NETWORK: Network);

    pub fn spawn_local_process<P>(process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: 'static,
    {
        NETWORK.with(|network| network.spawn_local_process(process))
    }

    pub fn spawn_process<P>(process: P) -> Arc<<P::Ports as Ports>::Handle>
    where
        P: Process,
        P::ExecFuture: Send + 'static,
    {
        NETWORK.with(|network| network.spawn_process(process))
    }

    pub fn spawn_local_task<F, T: 'static>(task: F) -> Task<T>
    where
        F: Future<Output = T> + 'static,
    {
        NETWORK.with(|network| network.spawn_local_task(task))
    }

    pub fn spawn_task<F, T: Send + 'static>(task: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
    {
        NETWORK.with(|network| network.spawn_task(task))
    }
}
