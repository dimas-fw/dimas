# dimas
[DiMAS](https://github.com/dimas-fw/dimas) - A framework for building a **Di**stributed **M**ulti **A**gent **S**ystem

A Distributed Multi Agent Systems is a system of independant working programs interchanging information,
that are running on several somehow connected computers (e.g. an ethernet network).

Dimas is tested on Windows (Version 10) and Linux (Ubuntu/Debian flavours) but should also run on `MacOS`.

This crate is on [crates.io](https://crates.io/crates/dimas) and can be
used by adding `dimas` to your dependencies in your project's `Cargo.toml`.

[DiMAS](https://github.com/dimas-fw/dimas) follows the semantic versioning principle with the enhancement, that until version 1.0.0
each new version may include breaking changes, which will be noticed in the changelog.

# Usage
DiMAS heavily relies on crates tokio, serde and zenoh.
So include the crate together with these crates in your dependencies in your `Cargo.toml`
```toml
[dependencies]
dimas = "0.0.2"
serde = "1.0"
serde_json = "1.0"
tokio = "1.35"
zenoh = "10.0.1-rc"
```
DiMAS needs an `async` runtime. So you have to define your `main` function as an `async` function:

```rust
#[tokio::main]
async fn main() {

}
```

You can find basic examples in [dimas/examples](https://github.com/dimas-fw/dimas/blob/main/examples/README.md)

Copyright © 2023 Stephan Kunz
