# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added support for `PCF8575` ([#1]).
- Added support for `PCA9538`.

[#1]: https://github.com/Rahix/port-expander/pull/1


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
