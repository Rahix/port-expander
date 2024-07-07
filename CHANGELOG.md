# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Fixed
- Fixed non-totem-pole devices not having `embedded_hal::{InputPin, OutputPin}`
  implementations due to a superfluous trait bound.  This fix was backported
  from 0.6.2.


## [0.5.0] - 2024-02-19
### Added
- Support for `PI4IOE5V6408` ([#17]).
- Added support for enabling pull-up/down resistors for port expander input
  pins ([#22]).
- Added public API to access the underlying port-expander of a pin
  (`pin.access_port_driver()`) and the register mask for the pin
  (`pin.pin_mask()`) ([#23]).

### Changed
- **BREAKING** Moved to `embedded-hal` 1.0 ([#16]).
- **BREAKING** Replaced `shared_bus::BusMutex` with our own custom
  `port_expander::PortMutex` trait ([#26]).  If you need support for custom
  mutex types, you now need to implement the latter one for your mutex.

[#16]: https://github.com/Rahix/port-expander/pull/16
[#17]: https://github.com/Rahix/port-expander/pull/17
[#22]: https://github.com/Rahix/port-expander/pull/22
[#23]: https://github.com/Rahix/port-expander/pull/23
[#26]: https://github.com/Rahix/port-expander/pull/26


## [0.4.1] - 2023-12-25
### Added
- Added support for `TCA6408A` ([#12]).
- Added support for `PCAL6408A` and `PCAL6416A` ([#13]).

[#12]: https://github.com/Rahix/port-expander/pull/12
[#13]: https://github.com/Rahix/port-expander/pull/13


## [0.4.0] - 2023-08-13
### Changed
- **BREAKING** Upgraded to `shared-bus` 0.3.


## [0.3.1] - 2023-08-13
### Added
- Added support for `MAX7321` ([#10]).

### Changed
- Upgraded to Rust Edition 2021.

[#10]: https://github.com/Rahix/port-expander/pull/10


## [0.3.0] - 2022-05-06
### Added
- Added support for `PCF8575` ([#1]).
- Added support for `PCA9538`.
- Added `into_output_high()` for totem-pole output drivers.  In contrast to
  `into_output()` this will immediately put the pin into a HIGH state, thus
  preventing a short glitch between setting direction and pin value ([#3]).
- Added support for setting the polarity inversion of pins.

### Changed
- `into_output()` for totem-pole output drivers now puts the pin into a LOW
  state without a glitch.  Previously, it would leave the pin in whatever state
  it was last in (= most often the HIGH state)  ([#3]).

### Fixed
- Fixed `read_multiple()` and `write_multiple()` not ensuring that all passed
  pins actually belong to the same port-expander chip ([#4]).

[#1]: https://github.com/Rahix/port-expander/pull/1
[#3]: https://github.com/Rahix/port-expander/pull/3
[#4]: https://github.com/Rahix/port-expander/pull/4


## [0.2.1] - 2021-04-26
### Added
- Added the `write_multiple()` and `read_multiple()` functions to set/get
  multiple pin-states in a single bus transaction.

### Changed
- The internal `PortDriver` trait was redesigned to better fit its requirements.


## [0.2.0] - 2021-04-24
### Changed
- `Pin::set_high()`, `Pin::set_low()`, and `Pin::toggle()` now take `&mut self`.

## [0.1.0] - 2021-04-24
Initial Release, with support for `PCA9536`, `PCA9555`, `PCF8574`, and
`PCF8574A`.

[Unreleased]: https://github.com/rahix/port-expander/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/rahix/port-expander/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/rahix/port-expander/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/rahix/port-expander/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/rahix/port-expander/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/rahix/port-expander/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/rahix/port-expander/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rahix/port-expander/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rahix/port-expander/releases/tag/v0.1.0
