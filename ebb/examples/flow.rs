#![feature(type_alias_impl_trait)]

use std::future::Future;
use std::time::{Duration, Instant};

use ebb::components::Interval;
use ebb::{Network, Process, Receiver};

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
                println!("Tick from Sample {} ({:?}, {:?}, {:?})", self.0, tick, elapsed, std::thread::current());
            }
        }
    }
}

pub fn main() {
    let network = Network::default();

    network.enter(|| {
        let interval =
            ebb::spawn_process(Interval::new(None, Duration::from_millis(1000)));
        let sample1 = ebb::spawn_process(SampleProcess(1));
        let sample2 = ebb::spawn_process(SampleProcess(2));
    
        &interval.output >> &sample1.interval;
        &interval.output >> &sample2.interval;
    });

    network.add_threads(1);
    network.complete();
}
