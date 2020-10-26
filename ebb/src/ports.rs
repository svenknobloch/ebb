use std::any::Any;

use crate::Network;

pub trait Ports: Sized {
    type Handle: Any;

    fn with_handle(network: &Network) -> (Self, Self::Handle);
}

impl<'a> From<&'a Network> for () {
    fn from(_: &'a Network) -> Self {
        ()
    }
}

impl Ports for () {
    type Handle = ();

    fn with_handle(_: &Network) -> (Self, Self::Handle) {
        ((), ())
    }
}
