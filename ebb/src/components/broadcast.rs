use std::future::Future;
use std::marker::PhantomData;

use futures::{SinkExt, StreamExt};

use crate::{Process, Receiver, Broadcaster};

#[derive(crate::Ports)]
pub struct Ports<T: Clone + 'static> {
    input: Receiver<T>,
    output: Broadcaster<T>,
}

pub struct Broadcast<T: Clone + 'static>(PhantomData<T>);

impl<T: Clone + 'static> Broadcast<T> {
    pub fn new() -> Self {
        Broadcast(Default::default())
    }
}

impl<T: Clone + 'static> Process for Broadcast<T>
{
    type Ports = Ports<T>;

    type ExecFuture = impl Future<Output = ()>;

    fn execute(self, mut ports: Self::Ports) -> Self::ExecFuture {
        async move {
            while let Some(value) = ports.input.next().await {
                ports.output.send(value).await.unwrap();
            }
        }
    }
}
