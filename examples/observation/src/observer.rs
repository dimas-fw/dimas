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
	new_limit: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {
		limit: 0u128,
		new_limit: 5u128,
	};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("observer")
		.config(&Config::default())?;

	// counter for occupancy
	let mut occupied_counter = 0u128;

	// create the observer for fibonacci
	#[allow(clippy::cognitive_complexity)]
	agent
		.observer()
		.topic("fibonacci")
		.control_callback(move |ctx, response| -> Result<()> {
			match response {
				ControlResponse::Accepted => {
					let limit = ctx.read()?.new_limit;
					info!("Accepted fibonacci up to {}", limit);
					ctx.write()?.limit = limit;
					ctx.write()?.new_limit += 1;
				}
				ControlResponse::Declined => {
					info!("Declined fibonacci up to {}", ctx.read()?.new_limit);
					ctx.write()?.limit = 0;
					ctx.write()?.new_limit = 5;
				}
				ControlResponse::Occupied => {
					info!("Service fibonacci is occupied");
					occupied_counter += 1;
					// cancel running request whenever 5 occupied messages arrived
					if occupied_counter % 5 == 0 {
						ctx.cancel_observe("fibonacci")?;
					}
				}
				ControlResponse::Canceled => {
					info!("Canceled fibonacci up to {}", ctx.read()?.limit);
				}
			};
			Ok(())
		})
		.response_callback(|_ctx, response| -> Result<()> {
			match response {
				ObservableResponse::Canceled(value) => {
					let msg = Message::new(value);
					let result: Vec<u128> = msg.decode()?;
					info!("canceled at {:?}", result);
				}
				ObservableResponse::Feedback(value) => {
					let msg = Message::new(value);
					let result: Vec<u128> = msg.decode()?;
					info!("received feedback {:?}", result);
				}
				ObservableResponse::Finished(value) => {
					let msg = Message::new(value);
					let result: Vec<u128> = msg.decode()?;
					info!("received result {:?}", result);
				}
			}
			Ok(())
		})
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(5);
	let mut counter = 0u128;
	agent
		.timer()
		.name("timer")
		.interval(interval)
		.callback(move |ctx| -> Result<()> {
			info!("Observation {counter}");
			let msg = FibonacciRequest {
				limit: ctx.read()?.new_limit,
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
