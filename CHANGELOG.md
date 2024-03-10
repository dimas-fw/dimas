# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Schema] - 2024-??-??

### Added

### Changed

### Fixed

### Removed


## [0.0.7] - 2024-??-?? <br>_Has breaking changes!!_

### Added
- Configuration via json5 file together with some new dedicated configuration methods

### Changed
- _The dedicated configuration method `Config::local()` returns an Error now_
- panic hooks in spawned tasks that they do not crash the application
- _Set on error handling: `Result`is always of type `std::result::Result<T, Box<dyn std::error::Error>>`_

### Fixed

### Removed


## [0.0.6] - 2024-03-03 <br>_Has breaking changes!!_

### Added
- using cargo vet as auditing tool

### Changed
- _Implemented error handling for callbacks which changes the signature of callbacks_
- _Separated `Response` for `Query` from `Message` for `Subscriber`_
- _Wrap zenoh `Sample` in messages which also changes the signature of callbacks_

### Fixed

### Removed
- Removed crate clap from examples
- calls to panic!, unwrap()'s, expect(...)'s and others

## [0.0.5] - 2024-02-29 <br>_Has breaking changes!!_

### Added
- Instrumentation level debug for communication activities
- Usage of closures for callbacks for all callbacks enabled

### Changed
- Optimized internal structure of `Agent`
- Re-exporting `Arc` & `RwLock` in prelude
- _Introduced `Message` which changes the signature of callbacks_
- Removed crate `bitcode` as exteral dependency by re-exporting it in prelude
- _Removed parameter `props` from callbacks, access now via `Context`_
- _Removed `Context<P>` from public api and replaced it with a thread safe `ArcContext<P>`_


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
- Moved montblanc benchmark (`benches/montblanc`) from this repo into separate repo `examples`

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
