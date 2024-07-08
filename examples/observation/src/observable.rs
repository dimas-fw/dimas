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

fn fibonacci(ctx: &Context<AgentProps>) -> Result<ResultResponse> {
	let n_2 = ctx.read()?.n_2;
	let n_1 = ctx.read()?.n_1;
	let next = n_2 + n_1;
	ctx.write()?.n_2 = n_1;
	ctx.write()?.n_1 = next;
	let result = Message::encode(&"done".to_string());
	Ok(ResultResponse::Finished(result.0))
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
		.control_callback(|ctx, msg| -> Result<ControlResponse> {
			let message: FibonacciRequest = msg.decode()?;
			// check if properties are still in initial state
			if ctx.read()?.limit == 0 && ctx.read()?.n_2 == 0 && ctx.read()?.n_1 == 1 {
				// accept
				info!("Accepting Fibonacci sequence up to {}", message.limit);
				ctx.write()?.limit = message.limit;
				Ok(ControlResponse::Accepted)
			} else {
				// decline
				info!("Declining Fibonacci sequence up to {}", message.limit);
				Ok(ControlResponse::Declined)
			}
		})
		.feedback_callback(|ctx| -> Result<Message> {
			let message = Message::encode(&"hello world".to_string());
			Ok(message)
		})
		.execution_function(fibonacci)
		.add()?;
	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
