//! `DiMAS` query example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use std::time::Duration;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

fn query_callback(_ctx: &Context<AgentProps>, response: Response) -> Result<()> {
	let message: u128 = response.decode()?;
	println!("Response 1 is '{message}'");
	Ok(())
}

fn query_callback2(response: Response) -> Result<()> {
	let message: u128 = response.decode()?;
	println!("Response 2 is '{message}'");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("query")
		.config(&Config::default())?;

	// create query for topic "query/1"
	agent
		.query()
		.topic("query1")
		.callback(query_callback)
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(4);
	let mut counter1 = 0i128;
	agent
		.timer()
		.name("timer1")
		.interval(interval)
		.callback(move |ctx| -> Result<()> {
			info!("Querying [{counter1}]");
			let message = Message::encode(&counter1);
			// querying with stored query
			ctx.get("query1", Some(message), None)?;
			counter1 += 4;
			Ok(())
		})
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(4);
	let delay = Duration::from_secs(1);
	let mut counter2 = 1i128;
	agent
		.timer()
		.name("timer2")
		.interval(interval)
		.delay(delay)
		.callback(move |ctx| -> Result<()> {
			info!("Querying [{counter2}]");
			let message = Message::encode(&counter2);
			// querying with ad-hoc query
			ctx.get("query2", Some(message), Some(&query_callback2))?;
			counter2 += 4;
			Ok(())
		})
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(4);
	let delay = Duration::from_secs(2);
	let mut counter3 = 2i128;
	agent
		.timer()
		.name("timer3")
		.interval(interval)
		.delay(delay)
		.callback(move |ctx| -> Result<()> {
			info!("Querying [{counter3}]");
			let message = Message::encode(&counter3);
			// querying with ad-hoc query & closure
			ctx.get(
				"query3",
				Some(message),
				Some(&|response| -> Result<()> {
					let message: u128 = response.decode()?;
					println!("Response 3 is '{message}'");
					Ok(())
				}),
			)?;
			counter3 += 4;
			Ok(())
		})
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(4);
	let delay = Duration::from_secs(3);
	let mut counter4 = 3i128;
	agent
		.timer()
		.name("timer4")
		.interval(interval)
		.delay(delay)
		.callback(move |ctx| -> Result<()> {
			info!("Querying [{counter4}]");
			let message = Message::encode(&counter4);
			// querying with stored query & closure
			ctx.get(
				"query1",
				Some(message),
				Some(&|response| -> Result<()> {
					let message: u128 = response.decode()?;
					println!("Response 4 is '{message}'");
					Ok(())
				}),
			)?;
			counter4 += 4;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
