//! `DiMAS` observation example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use observation::FibonacciRequest;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	limit: u128,
	n_2: u128,
	n_1: u128,
}

fn _fibonacci(ctx: &Context<AgentProps>) -> Result<u128> {
	let n_2 = ctx.read()?.n_2;
	let n_1 = ctx.read()?.n_1;
	let next = n_2 + n_1;
	ctx.write()?.n_2 = n_1;
	ctx.write()?.n_1 = next;
	Ok(next)
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {
		limit: 0u128,
		n_2: 0u128,
		n_1: 1u128,
	};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("observable")
		.config(&Config::default())?;

	// create the observable for fibonacci
	agent
		.observable()
		.topic("fibonacci")
		.callback(|ctx, msg| -> Result<ResponseType> {
			let message: FibonacciRequest = msg.decode()?;
			info!("Requesting Fibonacci sequence up to {}", message.limit);
			// check if properties are still in initial state
			if ctx.read()?.limit == 0 && ctx.read()?.n_2 == 0 && ctx.read()?.n_1 == 1 {
				// accept
				ctx.write()?.limit = message.limit;
				Ok(ResponseType::Accepted)
			} else {
				// decline
				Ok(ResponseType::Declined)
			}
		})
		.add()?;
	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
