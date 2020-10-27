#![feature(type_alias_impl_trait)]

use std::future::Future;
use std::time::{Duration, Instant};

use ebb::components::Interval;
use ebb::{Process, Receiver};

use futures::StreamExt;

#[derive(ebb::Ports)]
struct SampleProcessPorts {
    interval: Receiver<(Instant, Duration)>,
}

struct SampleProcess(usize);

impl Process for SampleProcess {
    type Ports = SampleProcessPorts;

    type ExecFuture = impl Future<Output = ()>;
    fn execute(self, mut ports: Self::Ports) -> Self::ExecFuture {
        async move {
            while let Some((tick, elapsed)) = ports.interval.next().await {
                println!("Tick from Sample {} ({:?}, {:?})", self.0, tick, elapsed);
            }
        }
    }
}

pub fn main() {
    let interval_handle =
        ebb::spawn_local_process(Interval::new(None, Duration::from_millis(1000)));
    let sample1_handle = ebb::spawn_local_process(SampleProcess(1));
    let sample2_handle = ebb::spawn_local_process(SampleProcess(2));

    interval_handle.output.connect(&sample1_handle.interval);
    interval_handle.output.connect(&sample2_handle.interval);

    ebb::complete();
}
