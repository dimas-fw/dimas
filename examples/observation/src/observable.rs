//! `DiMAS` observation example
//! Copyright © 2024 Stephan Kunz

use std::time::Duration;

// region:		--- modules
use dimas::prelude::*;
use observation::FibonacciRequest;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	limit: u128,
	sequence: Vec<u128>,
}

fn fibonacci(ctx: &Context<AgentProps>) -> Result<Message> {
	let limit = ctx.read()?.limit;
	let mut n_2 = 0;
	let mut n_1 = 1;
	for _ in 2..limit {
		let next = n_2 + n_1;
		n_2 = n_1;
		n_1 = next;
		ctx.write()?.sequence.push(next);
		// artificial time consumption
		std::thread::sleep(Duration::from_secs(1));
	}
	let result = Message::encode(&ctx.read()?.sequence);
	ctx.write()?.sequence.clear();
	info!("finished executor");
	Ok(result)
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {
		limit: 0u128,
		sequence: Vec::new(),
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
		.control_callback(|ctx, msg| -> Result<ControlResponse> {
			let message: FibonacciRequest = msg.decode()?;
			// check if properties are still in initial state
			if ctx.read()?.sequence.is_empty() {
				// accept
				info!("Accepting Fibonacci sequence up to {}", message.limit);
				ctx.write()?.limit = message.limit;
				// add first two elements
				ctx.write()?.sequence.push(0);
				ctx.write()?.sequence.push(1);
				Ok(ControlResponse::Accepted)
			} else {
				// decline
				info!("Declining Fibonacci sequence up to {}", message.limit);
				Ok(ControlResponse::Declined)
			}
		})
		.feedback_callback(|ctx| -> Result<Message> {
			info!("sending feedback");
			let seq = ctx.read()?.sequence.clone();
			let message = Message::encode(&seq);
			Ok(message)
		})
		.feedback_interval(Duration::from_secs(2))
		.execution_function(fibonacci)
		.add()?;
	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
