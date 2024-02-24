# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Schema] - 2024-??-??

### Added

### Changed

### Fixed

### Removed


## [0.0.5] - 2024-??-??

### Added
- Instrumentation level debug for communication activities

### Changed

### Fixed

### Removed


## [0.0.4] - 2024-02-23 <br>_Has breaking changes!!_

### Added
- Ability to store Publisher's and Query's in Agent's Context
- Introduce `tracing` support
- Agent::new_with_prefix() - as replacement for the old agent::new()
- Benchmarks
  - benches/montblanc/* - an implementation of the Montblanc benchmark for robots
  - benches/montblanc/tmux-robot.sh - a tmux script to run all of the robots nodes
  - benches/montblanc/tmux-workstation.sh - a tmux script to run all of the workstations nodes
- Examples
  - examples/tmux-exampes.sh - a tmux script to run all the examples

### Changed
- _Signature of Agent::new() now without 'prefix'_
- _Rename ad hoc methods in `Context` & `Communicator`_
- _A `Timer` needs a unique name_
- _Rework of `Context` which is now generic over the `Agent`'s properties_

### Fixed
- Cleanup dependencies


## [0.0.3] - 2024-02-08 <br>_Has breaking changes!!_

### Added
- More documentation
- Add Publisher, Query, Queryable, Subscriber, Timer and their Builders to public interface/prelude

### Changed
- _Changed crate for data serialization from 'bincode' to 'bitcode'_
- Moved zenoh into an internal dependency
- _Removed parameter `mode` in ad-hoc query through `Context`_

## [0.0.2] - 2024-01-28 <br>_Has breaking changes!!_

### Added
- README's & Documentation
- A pingpong example for roundtrip measurement

### Changed
- _Changed crate for data serialization from 'serde' to 'bincode' v2_
- _Reduce boilerplate code in callbacks_

### Removed
- Some unnecessary dependencies.


## [0.0.1] - 2024-01-21

### Added
- `Agent` with
  - Liveliness
  - Basic timer
  - Basic pub/sub functionality
  - Basic query/queryable functionality
- Examples: 
  - Liveliness
  - Publisher using Timer / Subscriber
  - Query using Timer / Queryable


## [0.0.0] - 2023-09-19

### Added
- Reservation of the crate name "dimas"
