# dimasctl
Control program for DiMAS, the **Di**stributed **M**ulti **A**gent **S**ystem framework.

## Installation
Can be installed from crates.io with

`cargo install dimasctl` 

or from within the root of the project directory with

`cargo install --path=./dimasctl`

Help is provided with

`dimasctl --help`

The [Zenoh Key Expression](https://zenoh.io/docs/manual/abstractions/#key-expression) logic is usable, do not forget to use quotes for the key expression to prevent shell enhancement of the `*`s.