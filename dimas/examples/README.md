# DiMAS Examples

Examples using DiMAS, the **Di**stributed **M**ulti **A**gent **S**ystem framework.

On Linux you can install `tmux` and in the `<workspace>` directory run
```shell
./tmux-examples.sh
```
or
```shell
./tmux-examples.sh --release
```
This will start all examples in parallel.

## [Liveliness Subscriber](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/liveliness/main.rs)

Implements a liveliness sender (token) and reciever (subscriber) in one program.
Starting this program twice in separate terminal windows with

```shell
cargo run --example liveliness
```

will show in both terminals an output similar to (will differ in the agent id)

```shell
Running `target/debug/liveliness`
2024-01-27T17:34:03.993964Z  INFO liveliness: liveliness: <agent-id> is alive
Number of agents is 2
```

The subscriber doesn't react on its own token.
You can also use the examples below, they all are liveliness token sender.

## Publisher/Subscriber

Implements a simple "Hello World!" Publisher/Subscriber pair

Run the [Publisher](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/publisher/main.rs)
in one terminal window with

```shell

cargo run --example publisher

```

and the [Subscriber](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/subscriber/main.rs)
in another terminal window with

```shell
cargo run --example subscriber
```

## [Queryable/Querier]

Implements a simple Qeryable/Querier pair, where the Querier does not wait for
a started Queryable, but continues querying.

Run the [Querier](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/querier/main.rs)
in one terminal window with

```shell

cargo run --example querier

```

and the [Queryable](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/queryable/main.rs)
in another terminal window with

```shell
cargo run --example queryable
```

## [Observable/Observer]

Implements a simple Observable/Observer pair, where the Observer does not wait
for a started Observable, but continues requesting an Observation.

Run the [Observer](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/observer/main.rs)
in one terminal window with

```shell
cargo run --example observer
```

and the [Observable](https://github.com/dimas-fw/dimas/blob/main/dimas/examples/observable/main.rs)
in another terminal window with

```shell
cargo run --example observable
```

