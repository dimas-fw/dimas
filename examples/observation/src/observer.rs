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
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps { limit: 10u128 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("observer")
		.config(&Config::default())?;

	// create the observer for fibonacci
	agent
		.observer()
		.topic("fibonacci")
		.callback(|ctx, msg| -> Result<()> {
			let message: ObservableResponse = msg.decode()?;
			info!("Observable response: {:?}", &message);
			match message {
				ObservableResponse::Accepted => ctx.write()?.limit += 10,
				ObservableResponse::Declined => ctx.write()?.limit += 1,
				_ => todo!(),
			};
			Ok(())
		})
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(10);
	let mut counter = 0u128;
	agent
		.timer()
		.name("timer")
		.interval(interval)
		.callback(move |ctx| -> Result<()> {
			info!("Observation {counter}");
			let msg = FibonacciRequest {
				limit: ctx.read()?.limit,
			};
			let message = Message::encode(&msg);
			ctx.observe("fibonacci", Some(message))?;
			counter += 1;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
