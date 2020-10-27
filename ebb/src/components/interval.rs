use std::future::Future;
use std::time::{Duration, Instant};

use futures::SinkExt;
use smol::Timer as TimerFuture;

use crate::{Broadcast, Process};

#[derive(crate::Ports)]
pub struct Ports {
    output: Broadcast<(Instant, Duration)>,
}

pub struct Interval {
    delay: Option<Duration>,
    interval: Duration,
}

impl Interval {
    pub fn new(delay: Option<Duration>, interval: Duration) -> Self {
        Self { delay, interval }
    }
}

impl Process for Interval {
    type Ports = Ports;

    type ExecFuture = impl Future<Output = ()>;

    fn execute(self, mut ports: Self::Ports) -> Self::ExecFuture {
        async move {
            let mut prev = Instant::now();

            if let Some(delay) = self.delay {
                TimerFuture::after(delay).await;
            }

            loop {
                let tick = Instant::now();
                let elapsed = tick - prev;
                ports.output.send((tick, elapsed)).await.unwrap();

                TimerFuture::after(self.interval).await;

                prev = tick;
            }
        }
    }
}
