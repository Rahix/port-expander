# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added support for `PCF8575` ([#1]).
- Added support for `PCA9538`.
- Added `into_output_high()` for totem-pole output drivers.  In contrast to
  `into_output()` this will immediately put the pin into a HIGH state, thus
  preventing a short glitch between setting direction and pin value ([#3]).

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

[Unreleased]: https://github.com/rahix/port-expander/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/rahix/port-expander/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rahix/port-expander/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rahix/port-expander/releases/tag/v0.1.0
