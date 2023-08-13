//! The device module contains the internals for each of the supported port expanders.
//!
//! In most cases you will not need anything from here explicitly, the exposed types at the root of
//! the crate should be enough.

pub mod max7321;
pub mod pca9536;
pub mod pca9538;
pub mod pca9555;
pub mod pcf8574;
pub mod pcf8575;
