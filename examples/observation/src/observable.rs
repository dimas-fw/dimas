//! `DiMAS` observation example
//! Copyright © 2024 Stephan Kunz

use std::time::Duration;

// region:		--- modules
use dimas::prelude::*;
use observation::FibonacciRequest;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	limit: u128,
	sequence: Vec<u128>,
}

fn fibonacci(ctx: &Context<AgentProps>) -> Result<Message> {
	let limit = ctx.read()?.limit;
	// clear any existing result
	ctx.write()?.sequence.clear();
	// create and add first two elements
	let mut n_2 = 0;
	ctx.write()?.sequence.push(n_2);
	let mut n_1 = 1;
	ctx.write()?.sequence.push(n_1);
	for _ in 2..limit {
		let next = n_2 + n_1;
		n_2 = n_1;
		n_1 = next;
		ctx.write()?.sequence.push(next);
		// artificial time consumption
		std::thread::sleep(Duration::from_millis(1000));
	}
	let sequence = ctx.read()?.sequence.clone();
	let result = Message::encode(&sequence);
	println!("result: {:?}", &sequence);
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
			// check wanted limit
			if message.limit > 2 && message.limit <= 20 {
				// accept
				println!("Accepting Fibonacci sequence up to {}", message.limit);
				ctx.write()?.limit = message.limit;
				Ok(ControlResponse::Accepted)
			} else {
				// decline
				println!("Declining Fibonacci sequence up to {}", message.limit);
				Ok(ControlResponse::Declined)
			}
		})
		.feedback_callback(|ctx| -> Result<Message> {
			let seq = ctx.read()?.sequence.clone();
			let message = Message::encode(&seq);
			println!("feedback: {:?}", &seq);
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
