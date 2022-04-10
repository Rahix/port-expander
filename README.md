`port-expander` [![crates.io page](https://img.shields.io/crates/v/port-expander.svg)](https://crates.io/crates/port-expander) [![docs.rs page](https://docs.rs/port-expander/badge.svg)](https://docs.rs/port-expander)
===============
This is a crate providing a common abstraction for I²C port-expanders.  This
abstraction is not necessarily the most performant, but it allows using the pins
just like direct GPIOs.  Because the pin types also implement the `embedded-hal`
digital IO traits, they can also be passed to further drivers downstream (e.g.
as a reset or chip-select pin).

## Example
```rust
// Initialize I2C peripheral from HAL
let i2c = todo!();

// A0: HIGH, A1: LOW, A2: LOW
let mut pca9555 = port_expander::Pca9555::new(i2c, true, false, false);
let pca_pins = pca9555.split();

let io0_0 = pca_pins.io0_0.into_output().unwrap();
let io1_5 = pca_pins.io0_1; // default is input

io0_0.set_high().unwrap();
assert!(io1_5.is_high().unwrap());
```

## Accessing multiple pins at the same time
Sometimes timing constraints mandate that multiple pin accesses (reading or
writing) happen at the same time.  The [`write_multiple()`][write-multiple] and
[`read_multiple()`][read-multiple] methods are designed for doing this.

[write-multiple]: https://docs.rs/port-expander/latest/port_expander/fn.write_multiple.html
[read-multiple]: https://docs.rs/port-expander/latest/port_expander/fn.read_multiple.html

## Supported Devices
The following list is what `port-expander` currently supports.  If you needs
support for an additional device, it should be easy to add.  It's best to take
a similar existing implementation as inspiration.  Contributions welcome!

- [`PCA9536`](https://docs.rs/port-expander/latest/port_expander/dev/pca9536/struct.Pca9536.html)
- [`PCA9555`](https://docs.rs/port-expander/latest/port_expander/dev/pca9555/struct.Pca9555.html)
- [`PCF8574A`](https://docs.rs/port-expander/latest/port_expander/dev/pcf8574/struct.Pcf8574a.html)
- [`PCF8574`](https://docs.rs/port-expander/latest/port_expander/dev/pcf8574/struct.Pcf8574.html)
- [`PCF8575`](https://docs.rs/port-expander/latest/port_expander/dev/pcf8575/struct.Pcf8575.html)

## Non-local sharing
`port-expander` uses the `BusMutex` from
[`shared-bus`](https://crates.io/crates/shared-bus) under the hood.  This means
you can also make the pins shareable across task/thread boundaries, given that
you provide an appropriate mutex type:

```rust
// Initialize I2C peripheral from HAL
let i2c = todo!();

// A0: HIGH, A1: LOW, A2: LOW
let mut pca9555: port_expander::Pca9555<std::sync::Mutex<_>> =
    port_expander::Pca9555::with_mutex(i2c, true, false, false);
let pca_pins = pca9555.split();
```

## License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
