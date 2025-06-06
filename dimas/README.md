# dimas

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) - A framework
for building **Di**stributed **M**ulti **A**gent **S**ystems

⚠️ WARNING ⚠️ : DiMAS is under active development,
so expect gaps between implementation and documentation.

A distributed multi agent system is a set of independant agents
that are widely distributed but somehow connected.
They are designed in a way that they can solve complex tasks by working together.

The system is characterised by

- a somewhat large and complex environment
- containing a set of (non agent) objects that can be perceived, created, moved,
modified or destroyed by the agents
- that changes over time due to external rules

with multiple agents operating in that environment which

- can perceive the environment to a limited extent
- have the possibility to communicate with some or all of the other agents
- have certain capabilities to influence the environment

This crate is available on [crates.io](https://crates.io/crates/dimas).

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) follows the semantic
versioning principle with the enhancement, that until version 1.0.0
each new minor version has breaking changes, while patches are non breaking
changes but may include enhancements.

## Usage

DiMAS uses the `tokio` runtime, you have to define your `main` function as an
`async` function. The declaration of tokio crate is not necessary, unless you use
tokio functionality within your implementations.

So include `dimas` runtime in the dependencies section of
your `Cargo.toml`.

Your `Cargo.toml` should include:

```toml
[dependencies]
dimas = "0.5"
```

It makes sense to return a `Result`, as most DiMAS `Agent`s functions return one.
DiMAS errors are of type `Box<dyn core::error::Error>` and must be thread safe.
DiMAS provides a type definition `Result<T>` to make life easier

DiMAS also provides a `main` attribute macro to create the runtime environment
and a `prelude` to import most used declarations.

A suitable main program skeleton may look like:

```rust
use dimas::prelude::*;

#[dimas::main]
async fn main() -> Result<()> {

    // your code
    // ...

    Ok(())
}
```

## Example

A very simple example consist at least of two agents, a `publisher` publishing messages
and a `subscriber` that is listening to those messages.

The `Cargo.toml` for this publisher/subscriber example should include

```toml
[dependencies]
dimas = version = "0.5"
```

### Publisher

The `publisher.rs` should look like this:

```rust,no_run
use dimas::prelude::*;
use core::time::Duration;

/// The Agent's properties
#[derive(Debug)]
struct AgentProps {
    counter: u128,
}

#[dimas::main]
async fn main() -> Result<()> {
    // create & initialize agents properties
    let properties = AgentProps { counter: 0 };

    // create an agent with the properties and default configuration
    let agent = Agent::new(properties)
       .config(&Config::default())?;

    // create publisher for topic "hello"
    agent
        .publisher()
        .topic("hello")
        .add()?;

    // use a timer for regular publishing of "hello" topic
    agent
        // get the TimerBuilder from the agent
        .timer()
        // set a name for the timer
        .name("timer")
        // every second
        .interval(Duration::from_secs(1))
        // the timers callback function as a closure
        .callback(
            |ctx| -> Result<()> {
                let counter = ctx
                    .read()?
                    .counter;
                // the message to send
                let text = format!("Hello World! [{counter}]");
                // just to see what will be sent
                println!("Sending '{}'", &text);
                // publishing with stored publisher for topic "hello"
                let message = Message::encode(&text);
                ctx.put("hello", message)?;
                // modify counter in properties
                ctx
                    .write()?
                    .counter += 1;
                Ok(())
            }
        )
        // finally add the timer to the agent
        // errors will be propagated to main
        .add()?;

    // start the agent
    agent.start().await?;
    Ok(())
}
```

### Subscriber

The `subscriber.rs` should look like this:

```rust,no_run
use dimas::prelude::*;

/// The Agent's properties
#[derive(Debug)]
pub struct AgentProps {}

async fn callback(_ctx: Context<AgentProps>, message: Message) -> Result<()> {
    let message: String = message.decode()?;
    println!("Received '{message}'");
    Ok(())
}

#[dimas::main]
async fn main() -> Result<()> {
    // create & initialize agents properties
    let properties = AgentProps {};

    // create an agent with the properties and default configuration
    let agent = Agent::new(properties)
        .config(&Config::default())?;

    // subscribe to "hello" messages
    agent
        // get the SubscriberBuilder from the agent
        .subscriber()
        //set wanted message topic (corresponding to publishers topic!)
        .topic("hello")
        // set the callback function for put messages
        .put_callback(callback)
        // finally add the subscriber to the agent
        // errors will be propagated to main
        .add()?;

    // start the agent
    agent.start().await?;
    Ok(())
}
```

## More examples

You can find some simple examples in [dimas-fw/dimas/examples](https://github.com/dimas-fw/dimas/blob/main/examples/README.md)
and more complex examples in [dimas-fw/examples](https://github.com/dimas-fw/examples/blob/main/README.md)

## Features

- unstable: Enables the unstable features.

## License

Licensed with a proprietary "NGMC" license, see [license file](https://github.com/dimas-fw/dimas/blob/main/LICENSE)

## Contribution

Any contribution intentionally submitted for inclusion in the work by you,
shall be licensed with the same "NGMC" license, without any additional terms or conditions.
