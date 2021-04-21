#![cfg_attr(not(test), no_std)]

mod common;
pub mod dev;

pub use common::I2cBus;
pub use common::Pin;
pub use common::Port;
pub use common::mode;

pub(crate) use common::Direction;
pub(crate) use common::PortDriver;

pub use dev::pca9536::Pca9536;
