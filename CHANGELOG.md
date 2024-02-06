# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Schema]

### Added

### Changed

### Fixed

### Removed


## [0.0.3] - 2024-??-?? <br>Has breaking changes!!

### Added
- More documentation

### Changed
- changed crate for data serialization from 'bincode' to 'bitcode'
- removed zenoh as external dependency
- removed parameter `mode` in ad-hoc query through `Context`

## [0.0.2] - 2024-01-28 <br>Has breaking changes!!

### Added
- README's & Documentation
- pingpong example for roundtrip measurement

### Changed
- changed crate for data serialization from 'serde' to 'bincode' v2
- reduce boilerplate code in callbacks

### Removed
- Some unnecessary dependencies.


## [0.0.1] - 2024-01-21

### Added
- agent with
  - liveliness
  - basic timer
  - basic pub/sub functionality
  - basic query/queryable functionality
- Examples: 
  - Liveliness
  - Timer / Publisher / Subscriber
  - Timer / Query / Queryable


## [0.0.0] - 2023-09-19

### Added
- Reservation of the crate name "dimas"
