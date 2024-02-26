# dimas

[DiMAS](https://github.com/dimas-fw/dimas/dimas) - A framework for building **Di**stributed **M**ulti **A**gent **S**ystems

A Distributed Multi Agent Systems is a system of independant working programs, interchanging information,
that are running on several somehow connected computers (e.g. an ethernet network).

This crate is on [crates.io](https://crates.io/crates/dimas).
`DiMAS` is tested on Windows (Version 10) and Linux (Debian flavours, AMD64 & aarch64) but should also run on `MacOS`.

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) follows the semantic versioning principle with the enhancement,
that until version 1.0.0 each new version may include breaking changes, which will be noticed in the changelog.

# Usage

DiMAS currently also needs to include the crates `bitcode` and `tokio`.
So include `dimas` together with these crates in the dependencies section of your `Cargo.toml`.

DiMAS uses features to have some control over compile time and the size of the binary. 
The feature `all`, including all available features, is a good point to start with.

```toml
[dependencies]
dimas = { version = "0.0.5", features = ["all"] }
bitcode = "0.5"
tokio = { version = "1", features = ["macros"] }
```

DiMAS needs an `async` runtime. So you have to define your `main` function as an `async` function.
It also makes sense to return a `Result` as some functions return one. DiMAS prelude provides a simplified `Result` type for that.

```rust
use dimas::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {

	Ok(())
}
```

## Example

A very simple example consist at least of two agents, a `publisher` publishing messages 
and a `subscriber` that is listening to those messages.

Your `Cargo.toml` should include

```toml
[dependencies]
dimas = { version = "0.0.5", features = ["timer", "subscriber"] }
bitcode = "0.5"
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

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), properties);

	// create publisher for topic "hello"
	agent
		.publisher()
		.msg_type("hello")
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
			|ctx, props| {
				let counter = props
					.read()
					.unwrap()
					.counter
					.to_string();
				// the message to send
				let text = "Hello World! [".to_string() + &counter + "]";
				// just to see what will be sent
				println!("Sending '{}'", &text);
				// publishing with stored publisher for topic "hello"
				let _ = ctx.put_with("hello", text);
				// modify counter in properties
				props
					.write()
					.unwrap()
					.counter += 1;
			}
		)
		// finally add the timer to the agent
		// errors will be propagated to main
		.add()?;

	// run the agent
	agent.start().await;
	Ok(())
}
```

#### Subscriber

The `subscriber.rs` should look like this:

```rust,no_run
use dimas::prelude::*;

#[derive(Debug)]
pub struct AgentProps {}

fn callback(_ctx: &Arc<Context<AgentProps>>, _props: &Arc<RwLock<AgentProps>>, message: &Message) {
	let message: String =	bitcode::decode(message).unwrap();
	println!("Received '{}'", &message);
}

#[tokio::main]
async fn main() -> Result<()> {
	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), properties);

	// subscribe to "hello" messages
	agent
		// get the SubscriberBuilder from the agent
		.subscriber()
    	//set wanted message topic (corresponding to publishers topic!)
		.msg_type("hello")
    	// set the callback function
		.put_callback(callback)
    	// finally add the subscriber to the agent
    	// errors will be propagated to main
		.add()?;

	// run the agent
	agent.start().await;
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
- `liveliness`: Enables liveliness features sending tokens and listening for them.
- `publisher`: Enables to store `Publisher`'s within the `Agent`'s `Context`.
- `query`: Enables to store `Query`'s within the `Agent`'s `Context`.
- `queryable`: Enables to store `Queryable`'s within the `Agent`'s `Context`.
- `subscriber`: Enables to store `Subsciber`'s within the `Agent`'s `Context`.
- `timer`: Enables to store `Timer`'s within the `Agent`'s `Context`.
