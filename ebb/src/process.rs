use std::future::Future;

use crate::Ports;

pub trait Process: 'static {
    type Ports: Ports;

    type ExecFuture: Future<Output = ()>;
    fn execute(self, ports: Self::Ports) -> Self::ExecFuture;
}
