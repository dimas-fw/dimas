//! Copyright © 2023 Stephan Kunz
#![crate_type = "lib"]
#![crate_name = "dimas"]
#![warn(missing_docs)]

/*! [DiMAS](https://github.com/dimas-fw) /dimas/ is a framework to develop distributed multi agent systems.

A Distributed Multi Agent Systems is a system of independant working programms interchanging information, 
that are running on several somehow connected computers (e.g. an ethernet network).

DiMAS is tested on Windows (Version 10) and Linux (Ubuntu/Debian flavours) but should also run on MacOS.

# Usage

This crate is [on crates.io](https://crates.io/crates/dimas) and can be
used by adding `dimas` to your dependencies in your project's `Cargo.toml`.

DiMAS follows the semantic versioning principle with the enhancement, that until version 1.0.0
each new version may include breaking changes, which will be noticed in the changelog.

```toml
[dependencies]
dimas = "0.0.1"
```

*/

// region:    --- modules
mod agent;
mod com;
mod config;
mod context;
mod error;
mod message;
#[cfg(feature = "timer")]
mod timer;
mod utils;

// the public interface
pub mod prelude;
// endregion: --- modules
