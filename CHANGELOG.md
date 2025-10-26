# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Schema] - 2025-??-??

### Added

### Changed

### Fixed

### Removed

## [0.5.1] - 2025-10-26

### Added
- README for examples

### Changed
- minimum Rust version to 1.88.0
- updated dependencies

### Fixed
- Clippy complaints
- tmux script updated

## [0.5.0] - 2024-06-06

### Added
- Example for liveliness

### Changed
- Made liveliness stable
- Formatting

### Fixed
- Multiple incoming liveliness messages from the same agent
- Error message when zenoh communicator received an error  

### Removed
- Some forgotten dbg!(...) statements

## [0.4.2] - 2025-05-15

### Changed
- Updated dependencies
- Minimum Rust version adapted to 1.85 due to dependencies

### Fixed
- Some missing comments
- Solved complaints of Clippy
- Missing std lib for tokio

## [0.4.1] - 2024-11-13

### Fixed

- make multi session communication work
- bug in configuration directory

## [0.4.0] - 2024-10-31

### Added

- timeout handling to `Querier` and `Observer`
- macro `dimas::main`
- module for time related stuff
- traits `Communicator`, `Observer`, `Publisher`, `Querier`, `Responder` in dimas-com
- multi session/protocol communication with traits
  `SingleSessionCommunicator` & `MultiSessionCommunicator`

### Changed

- use zenoh release 1.0.0
- modules structured
- method for getting a `QuerierBuilder` from an `Agent` to `querier()`
- error handling without `thiserror`
- usage of `tokio::main` macro replaced with `dimas::main` macro
- replaced `tokio::time::Duration` with `core::time::Duration`
- replaced `std::` with `alloc::` and `core::` where possible

### Fixed

- operation state management for `Liveliness`, `Observable`, `Observer`,
`Publisher`, `Querier`, `Queryabler`, `Subscriber`
- console output of `dimasctl`

### Removed

- example for router (is now in repository [examples](https://github.com/dimas-fw/examples/tree/main/router))
- no longer needed clippy allows
- setting `undeclare_on_drop`
- necessity to declare crate `tokio` in applications

## [0.3.0] - 2024-10-03 _Has breaking changes!!_

### Added

- re-export of `tokio::time::Duration` in `dimas::prelude`
- feature `unstable`, which encapsulates zenoh unstable feature
- additional settings in: `PublisherBuilder`, `SubscriberBuilder`,
  `Querier`, `QueryableBuilder`. `ObserableBuilder`. `Observer`
- attribute macro for `main` function: `#[dimas::main(...)]`

### Changed

- bumped minimum rust version to 1.81
- replaced `std::` with `core::` in some places
- renamed `Query` to `Querier` `QueryBuilder` to `QuerierBuilder`
  to differentiate from data structure
- callbacks get a `Context<P>` instead of a `&Context<P>`
- internal structure
- renamed `ObservableBuilder::execution_function` to `execution_callback`
- renamed `ObserverBuilder::response_callback` to `result_callback`
- async callbacks for `LivelinessSubscriber`, `Observable`, `Observer`, `Querier`,
  `Queryable`, `Subscriber`
  (this makes the usage of closures for those callbacks difficult
  until async closures are stable)
- changed trait `ContextAbstraction` from generic to associated type

### Fixed

- some clippy hints
- changes in zenoh api

## [0.2.5] - 2024-09-21 _Has breaking changes!!_

### Changed

- predeclaration of query key expression
- sequentialize initial liveliness query and liveliness subscriber

### Fixed

- changes in zenoh api

## [0.2.4] - 2024-08-31

### Fixed

- logic of observation cancelation
- missing checks of Mutexes

## [0.2.3] - 2024-08-21 _Has breaking changes!!_

### Changed

- refactored observer and observable
- replaced std::sync::mpsc with tokio::sync::mpsc

### Fixed

- change of zenoh config syntax in provided config files
- adopted to latest zenoh syntax changes

## [0.2.2] - 2024-07-27

### Added

- incomplete observable and observer with example

### Changed

- internal restructuring of the builders
- bump to zenoh 1.0.0-alpha
- priority of lints

## [0.2.1] - 2024-06-06

### Added

- documentation how to install dimasctl & dimasmon

### Changed

- zenoh version updated to 0.11
- replaced &Option\<T\> with Option\<&T\>

### Fixed

- query/queryable with `Message`

## [0.2.0] - 2024-05-29 _Has breaking changes!!_

### Added

- dimasctl: binary to control DiMAS entities with commands
  - `scout`
  - `list`
  - `ping <target>`
  - `set-state <OperationState>`
  - `shutdown <target>`
- dimasmon: binary to monitor DiMAS entities (not yet usable!)
- dimas-commands with functions for dimasctl & dimasmon

### Changed

- splitted `dimas` into several crates
  - dimas-core: for core functionalities
  - dimas-com: for `Communicator` & `Messages`
  - dimas-config: for `Config`
- signature of `Agent::config(self, ...)` -> `Config` is now passed as reference
- signature of Query::get(), Communicator::get(): added an `Option<&Message>`
- encoding of types `Message` & `Response` now explicit in callbacks
- renaming & restructuring

### Removed

- low_latency configuration
- features
- method `Communicator::create_publisher(&self, ...)`

## [0.1.1] - 2024-05-01

### Added

- Constructors for Publisher, Query, Queryable, Subscriber, Timer

### Changed

- renamed module 'liveliness_subscriber' to 'liveliness'

### Fixed

- compile & doc warnings

## [0.1.0] - 2024-04-25

### Added

- Enhance Documentation
- Additional features for Publisher, Subscriber, Query & Queryable

### Changed

use core::time::Duration;

- MSRV bumped to '1.77' due to Mutex::clear_poison()
- Bumped version of 'zenoh' to '0.11.0-rc'
- Bumped version of 'bitcode' to '0.6'
- cleanup coding

### Fixed

- tracing initialisation now fits to zenohs usage of tracing crate

## [0.0.8] - 2024-03-22 _Has breaking changes!!_

### Added

- Documentation
- Builders 'add' methods now return the possibly previously added item
  for that key expression
- Agent can have a (non unique) name
- QoS for Publisher & Subscriber

### Changed

- _Name of Builder methods to set only topics name chanched to 'topic(..)'_
- _Removed tokio 'flavor=current_thread' due to changes in zenoh_
- _Creation of Agent now uses a builder pattern_

### Fixed

- Broken documentaton on docs.rs
- Same naming scheme for TimerBuilder as for other builders

## [0.0.7] - 2024-03-17 _Has breaking changes!!_

### Added

- Configuration via json5 file together with some new dedicated configuration methods
- ArcContext gives access to Builders

### Changed

- _The dedicated configuration method `Config::local()` returns an Error now_
- Panic hooks in spawned tasks that prevent tasks from crashing,
  they will be restarted instead
- Set on error handling:
  `Result`is always of type `std::result::Result<T, Box<dyn std::error::Error>>`
- All of the Builders are implemented with type state pattern
- ArcContext is now a regular struct not only a type
- updated internal dependencies of zenoh & bitcode

## [0.0.6] - 2024-03-03 _Has breaking changes!!_

### Added

- using cargo vet as auditing tool

### Changed

- _Implemented error handling for callbacks which changes the signature of callbacks_
- _Separated `Response` for `Query` from `Message` for `Subscriber`_
- _Wrap zenoh `Sample` in messages which also changes the signature of callbacks_

### Removed

- Removed crate clap from examples
- calls to panic!, unwrap()'s, expect(...)'s and others

## [0.0.5] - 2024-02-29 _Has breaking changes!!_

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

## [0.0.4] - 2024-02-23 _Has breaking changes!!_

### Added

- Ability to store Publisher's and Query's in Agent's Context
- Introduce `tracing` support
- Agent::new_with_prefix() - as replacement for the old Agent::new()
- Benchmarks
  - benches/montblanc/* - an implementation of the Montblanc benchmark for robots
  - benches/montblanc/tmux-robot.sh - a tmux script to run all of the robots nodes
  - benches/montblanc/tmux-workstation.sh,
    a tmux script to run all of the workstations nodes
- Examples
  - examples/tmux-exampes.sh - a tmux script to run all the examples

### Changed

- _Signature of Agent::new() now without 'prefix'_
- _Rename ad hoc methods in `Context` & `Communicator`_
- _A `Timer` needs a unique name_
- _Rework of `Context` which is now generic over the `Agent`'s properties_
- Moved montblanc benchmark (`benches/montblanc`) from this repo
  into separate repo `examples`

### Fixed

- Cleanup dependencies

## [0.0.3] - 2024-02-08 _Has breaking changes!!_

### Added

- More documentation
- Add Publisher, Query, Queryable, Subscriber, Timer and their Builders
  to public interface/prelude

### Changed

- _Changed crate for data serialization from 'bincode' to 'bitcode'_
- Moved zenoh into an internal dependency
- _Removed parameter `mode` in ad-hoc query through `Context`_

## [0.0.2] - 2024-01-28 _Has breaking changes!!_

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
