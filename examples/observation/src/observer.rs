//! `DiMAS` observation example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use observation::FibonacciRequest;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	limit: u128,
	new_limit: u128,
	occupied_counter: u8,
}

async fn control_response(ctx: Context<AgentProps>, response: ControlResponse) -> Result<()> {
	match response {
		ControlResponse::Accepted => {
			let limit = ctx.read()?.new_limit;
			println!("Accepted fibonacci up to {limit}");
			ctx.write()?.limit = limit;
			ctx.write()?.new_limit += 1;
		}
		ControlResponse::Declined => {
			println!("Declined fibonacci up to {}", ctx.read()?.new_limit);
			ctx.write()?.limit = 0;
			ctx.write()?.new_limit = 5;
		}
		ControlResponse::Occupied => {
			println!("Service fibonacci is occupied");
			let occupied_counter = ctx.read()?.occupied_counter + 1;
			// cancel running request whenever 5 occupied messages arrived
			if occupied_counter % 5 == 0 {
				ctx.cancel_observe("fibonacci")?;
				ctx.write()?.occupied_counter = 0;
			} else {
				ctx.write()?.occupied_counter = occupied_counter;
			}
		}
		ControlResponse::Canceled => {
			println!("Canceled fibonacci up to {}", ctx.read()?.limit);
		}
	};
	Ok(())
}

async fn response(ctx: Context<AgentProps>, response: ObservableResponse) -> Result<()> {
	match response {
		ObservableResponse::Canceled(value) => {
			let msg = Message::new(value);
			let result: Vec<u128> = msg.decode()?;

			println!("Canceled at {result:?}");
		}
		ObservableResponse::Feedback(value) => {
			let msg = Message::new(value);
			let result: Vec<u128> = msg.decode()?;
			let limit = ctx.read()?.limit;
			if result.len() <= limit as usize {
				println!("Received feedback {result:?}");
			} else {
				println!("Wrong feedback {result:?}");
			}
		}
		ObservableResponse::Finished(value) => {
			let msg = Message::new(value);
			let result: Vec<u128> = msg.decode()?;
			let limit = ctx.read()?.limit;
			if result.len() == limit as usize {
				println!("Received result {result:?}");
			} else {
				println!("Wrong result {result:?}");
			}
		}
	}
	Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {
		limit: 0u128,
		new_limit: 5u128,
		occupied_counter: 0u8,
	};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("observer")
		.config(&Config::default())?;

	// create the observer for fibonacci
	agent
		.observer()
		.topic("fibonacci")
		.control_callback(control_response)
		.result_callback(response)
		.add()?;

	// timer for next observation
	let interval = Duration::from_secs(5);
	agent
		.timer()
		.name("timer")
		.interval(interval)
		.callback(move |ctx| -> Result<()> {
			let limit = ctx.read()?.new_limit;
			println!("request fibonacci up to {limit}");
			let msg = FibonacciRequest {
				limit,
			};
			let message = Message::encode(&msg);
			ctx.observe("fibonacci", Some(message))?;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
