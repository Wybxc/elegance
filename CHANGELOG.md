# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- markdownlint-disable MD024 -->

## [Unreleased]

## [0.3.4](https://github.com/Wybxc/elegance/compare/v0.3.3...v0.3.4) - 2025-12-13

### Changed

- `Render` trait implementations for `String` and `OsString` now use `std::convert::Infallible` as the error type, as these writers cannot fail.

## [0.3.3](https://github.com/Wybxc/elegance/compare/v0.3.2...v0.3.3) - 2025-10-17

### Changed

- Dependency updates.

## [0.3.2](https://github.com/Wybxc/elegance/compare/v0.3.1...v0.3.2) - 2025-03-07

### Added

- `unicode-width` feature for accurate Unicode character width calculation (enabled by default).

## [0.3.1](https://github.com/Wybxc/elegance/compare/v0.3.0...v0.3.1) - 2025-03-07

### Changed

- `Printer::text_owned` now accepts any type implementing `Into<S>` instead of just `S`.

## [0.3.0](https://github.com/Wybxc/elegance/compare/v0.2.1...v0.3.0) - 2025-03-07

### Added

- Generic string storage type `S` for `Printer<'a, R, S>`.
- `Printer::new_with` constructor for custom string types.
- `text_owned` method to handle owned strings and types convertible to `S`.
- Public `string` module exposing `CowString` for borrowed/owned text handling.

### Removed

- `Printer::text` no longer accepts owned types; use `text_owned` instead.

## [0.2.1](https://github.com/Wybxc/elegance/compare/v0.2.0...v0.2.1) - 2025-02-15

### Changed

- Added crates.io categories and keywords for better discoverability.

## [0.2.0](https://github.com/Wybxc/elegance/compare/v0.1.0...v0.2.0) - 2025-02-15

### Added

- `cgroup` method for consistent indented groups (all breaks align after first wrap).
- `igroup` method for inconsistent indented groups (breaks continue flowing after first wrap).

### Removed

- `group` method no longer exists; use `cgroup` or `igroup` instead (they require explicit `consistent` parameter).

### Changed

- Improved line-breaking algorithm to handle multiple consecutive breaks and spacing more predictably.
