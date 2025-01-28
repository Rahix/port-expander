//! The device module contains the internals for each of the supported port expanders.
//!
//! In most cases you will not need anything from here explicitly, the exposed types at the root of
//! the crate should be enough.

pub mod max7321;
pub mod mcp23x17;
pub mod pca9536;
pub mod pca9538;
pub mod pca9554;
pub mod pca9555;
pub mod pca9702;
pub mod pcal6408a;
pub mod pcal6416a;
pub mod pcf8574;
pub mod pcf8575;
pub mod pi4ioe5v6408;
pub mod tca6408a;
