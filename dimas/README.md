# dimas

[DiMAS](https://github.com/dimas-fw/dimas/dimas) - A framework for building **Di**stributed **M**ulti **A**gent **S**ystems

A Distributed Multi Agent Systems is a system of independant working programs, interchanging information,
that are running on several somehow connected computers (e.g. an ethernet network).

DiMAS is tested on Windows (Version 10) and Linux (Ubuntu/Debian flavours) but should also run on MacOS.

This crate is on [crates.io](https://crates.io/crates/dimas) and can be
used by adding `dimas` to your dependencies in your project's `Cargo.toml`.

[DiMAS](https://github.com/dimas-fw/dimas/tree/main/dimas) follows the semantic versioning principle with the enhancement, that until version 1.0.0
each new version may include breaking changes, which will be noticed in the changelog.

# Usage

DiMAS currently also needs to include the crates `bincode` and `tokio`.
So include `dimas` together with these crates in your dependencies section in your `Cargo.toml`.

DiMAS uses features to have some control over compile time and the size of the binary. 
The feature `all`, including all available features, is a good point to start with.

```toml
[dependencies]
dimas = { version = "0.0.3", features = ["all"] }
bincode = "2.0.0-rc.3"
tokio = "1.35"
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
dimas = { version = "0.0.3", features = ["timer", "publisher", "subscriber"] }
bincode = "2.0.0-rc.3"
tokio = "1.35"
```

### Publisher

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

	// create an agent with the properties and the prefix "example"
	let mut agent = Agent::new(Config::default(), "example", properties);

	// timer for regular publishing
	agent
    // get the TimerBuilder from the agent
    .timer()
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
			  // publishing with ad-hoc publisher as topic "hello"
			  let _ = ctx.publish("hello", text);
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

### Subscriber

The `subscriber.rs` should look like this:

```rust,no_run
use dimas::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct AgentProps {}

fn callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let config = bincode::config::standard();
	let (message, _len): (String, usize) =
		bincode::decode_from_slice(message, config).expect("should not happen");
	println!("Received '{}'", &message);
}

#[tokio::main]
async fn main() -> Result<()> {
	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix "example"
	let mut agent = Agent::new(Config::default(), "example", properties);

  // listen for 'hello' messages
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

  // start the agent
	agent.start().await;
  Ok(())
}
```

You can find more examples in [dimas/examples](https://github.com/dimas-fw/dimas/blob/main/examples/README.md)

## Feature flags

DiMAS uses a set of feature flags to reduce the amount of compiled code. 
It is necessary to enable all those features you want to use with your `Agent`.

- `all`: Enables all the features listed below. It's a good point to start with.
- `liveliness`: Enables liveliness features sending tokens and listening for them.
- `publisher`: Enables publishing of messages.
- `query`: Enables querying the queryables.
- `queryable`: Enables listening for queries and answering them.
- `subscriber`: Enables listening for publisher messages.
- `timer`: Enables a timer feature, typically used with publishers and/or queries.
