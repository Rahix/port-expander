#![cfg_attr(not(test), no_std)]

mod common;
pub mod dev;
mod pin;
mod bus;

pub use common::mode;
pub use bus::I2cBus;
pub use common::Port;
pub use pin::Pin;

pub(crate) use common::Direction;
pub(crate) use common::PortDriver;

pub use dev::pca9536::Pca9536;
