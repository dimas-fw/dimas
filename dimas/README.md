# dimas

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) - A framework for building **Di**stributed **M**ulti **A**gent **S**ystems

⚠️ WARNING ⚠️ : DiMAS is under active development, so expect gaps between implementation and documentation. 

A distributed multi agent system is a set of independant agents that are widely distributed but somehow connected.
They are designed in a way that they can solve complex tasks by working together.

The system is characterised by
- a somewhat large and complex environment
- containing a set of (non agent) objects that can be perceived, created, moved, modified or destroyed by the agents
- that changes over time due to external rules

with multiple agents operating in that environment which
- can percieve the environment to a limited extent
- have the possibility to communicate with some or all of the other agents
- have certain capabilities to influence the environment


This crate is available on [crates.io](https://crates.io/crates/dimas).

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) follows the semantic versioning principle with the enhancement,
that until version 1.0.0 each new version may include breaking changes, which will be noticed in the changelog.

# Usage

DiMAS needs an `async` runtime. So you have to define your `main` function as an `async` function.

So include `dimas` together with an async runtime in the dependencies section of your `Cargo.toml`.
As DiMAS uses `tokio` as async runtime, so preferably use `tokio` for your application. 
Ensure that you use a multi-threaded runtime, otherwise dimas will panic.

DiMAS uses features to have some control over compile time and the size of the binary. 
The feature `all`, including all available features, is a good point to start with.

So your `Cargo.toml` should include:

```toml
[dependencies]
dimas = { version = "0.1.0", features = ["all"] }
tokio = { version = "1", features = ["macros"] }
```

It also makes sense to return a `Result`, as some functions may return one.
DiMAS errors are always of type `Box<dyn std::error::Error>` and should be thread safe. 
DiMAS provides a type definition `Result<T>` to make life easier

A suitable main programm skeleton may look like:

```rust
use dimas::prelude::*;

#[tokio::main]
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
dimas = { version = "0.1.0", features = ["timer", "subscriber"] }
tokio = { version = "1",features = ["macros"] }
```

#### Publisher

The `publisher.rs` should look like this:

```rust,no_run
use dimas::prelude::*;
use std::time::Duration;

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and default configuration
	let mut agent = Agent::new(properties)
		.config(Config::default())?;

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
				ctx.put_with("hello", text)?;
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

#### Subscriber

The `subscriber.rs` should look like this:

```rust,no_run
use dimas::prelude::*;

#[derive(Debug)]
pub struct AgentProps {}

fn callback(_ctx: &ArcContext<AgentProps>, message: Message) -> Result<()> {
	let message: String =	message.decode()?;
	println!("Received '{}'", message);
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and default configuration
	let mut agent = Agent::new(properties)
		.config(Config::default())?;

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

#### More examples
You can find some simple examples in [dimas-fw/dimas/examples](https://github.com/dimas-fw/dimas/blob/main/examples/README.md)
and more complex examples in [dimas-fw/examples](https://github.com/dimas-fw/examples/blob/main/README.md)

## Feature flags

DiMAS uses a set of feature flags to minimize the size of an agent. 
It is necessary to enable all those features you want to use within your `Agent`.

- `all`: Enables all the features listed below. It's a good point to start with.
- `liveliness`: Enables listening and reacting on liveliness tokens. Sending tokens is always possible.
- `publisher`: Enables to store `Publisher` within the `Agent`s `Context`.
- `query`: Enables to store `Query`s within the `Agent`s `Context`.
- `queryable`: Enables to store `Queryable`s within the `Agent`s `Context`.
- `subscriber`: Enables to store `Subscriber` within the `Agent`s `Context`.
- `timer`: Enables to store `Timer` within the `Agent`s `Context`.
