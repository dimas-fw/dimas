# DiMAS Examples

Examples using DiMAS, the **Di**stributed **M**ulti **A**gent **S**ystem framework.

## [Liveliness Token/Subscriber](https://github.com/dimas-fw/dimas/tree/main/examples/liveliness)

Implements a liveliness sender (token) and reciever (subscriber) in one program.
Starting this program twice in separate terminal windows with

```shell
cargo run --bin liveliness
```

will show in both terminals an an output similar to (will differ in the agent id)

```shell
Running `target/debug/liveliness`
2024-01-27T17:34:03.993964Z  INFO liveliness: liveliness: <agent-id> is alive
Number of agents is 3
```

The subscriber doesn't see its own token.

## [Publisher/Subscriber](https://github.com/dimas-fw/dimas/tree/main/examples/pubsub)

Implements a simple "Hello World!" Publisher/Subscriber pair

Run the publisher in one terminal window with

```shell

cargo run --bin publisher

```

and the subscriber in another terminal window with

```shell
cargo run --bin subscriber
```

## [Ping/Pong](https://github.com/dimas-fw/dimas/tree/main/examples/pingpong)

Implements a Ping/Pong pair that measures the roundtrip time. The Ping does not
wait for a started Pong, but continues pinging.

Run the ping'er in one terminal window with

```shell

cargo run --bin ping

```

and the pong'er in another terminal window with

```shell
cargo run --bin pong
```

## [Queryable/Querier](https://github.com/dimas-fw/dimas/tree/main/examples/queries)

Implements a simple Qeryable/Querier pair, where the Querier does not wait for
a started Queryable, but continues querying.

Run the query in one terminal window with

```shell

cargo run --bin query

```

and the queryable in another terminal window with

```shell
cargo run --bin queryable
```

## [Observable/Observer](https://github.com/dimas-fw/dimas/tree/main/examples/observation)

Implements a simple Observable/Observer pair, where the Observer does not wait
for a started Observable, but continues requesting an Observation.

Run the observer in one terminal window with

```shell
cargo run --bin observer
```

and the observable in another terminal window with

```shell
cargo run --bin observable
```
