use std::any::Any;

use crate::NetworkConfig;

pub trait Ports: Sized {
    type Handle: Any;

    fn handle(&self) -> Self::Handle;
    fn create(config: &NetworkConfig) -> Self;
}

impl Ports for () {
    type Handle = ();

    fn handle(&self) -> Self::Handle {
        Default::default()
    }

    fn create(_: &NetworkConfig) -> Self {
        Default::default()
    }
}
