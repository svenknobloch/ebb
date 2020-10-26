use std::future::Future;

use crate::{Network, Ports};

pub trait Process: 'static {
    type Ports: Ports;

    type ExecFuture: Future<Output = ()>;
    fn execute(self, network: &Network, ports: Self::Ports) -> Self::ExecFuture;
}
