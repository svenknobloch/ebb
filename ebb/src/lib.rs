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
