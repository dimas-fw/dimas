//! `DiMAS` query example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use std::time::Duration;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

fn query_callback(_ctx: &ArcContext<AgentProps>, response: Response) -> Result<()> {
	let message: u128 = response.decode()?;
	println!("Response is '{message}'");
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

	// create publisher for topic "ping"
	agent
		.query()
		.topic("query")
		.callback(query_callback)
		.add()?;

	// timer for regular querying
	let duration = Duration::from_secs(1);
	let mut counter = 0i128;
	agent
		.timer()
		.name("timer")
		.interval(duration)
		.callback(move |ctx| -> Result<()> {
			info!("Querying [{counter}]");
			// querying with stored query
			ctx.get_with("query")?;
			counter += 1;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
