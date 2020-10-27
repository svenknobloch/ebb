use std::future::Future;
use std::marker::PhantomData;

use futures::{SinkExt, StreamExt};

use crate::{Process, Receiver, Sender};

#[derive(crate::Ports)]
pub struct Ports<T: 'static> {
    input: Receiver<T>,
    output: Sender<T>,
}

pub struct Inspect<T: 'static, F: 'static> {
    phantom: PhantomData<T>,
    f: F,
}

impl<T, F> Inspect<T, F>
where
    F: Fn(&T),
{
    pub fn new(f: F) -> Self {
        Inspect {
            phantom: Default::default(),
            f,
        }
    }
}

impl<T, F> Process for Inspect<T, F>
where
    F: Fn(&T),
{
    type Ports = Ports<T>;

    type ExecFuture = impl Future<Output = ()>;

    fn execute(self, mut ports: Self::Ports) -> Self::ExecFuture {
        async move {
            while let Some(value) = ports.input.next().await {
                (self.f)(&value);
                ports.output.send(value).await.unwrap();
            }
        }
    }
}
