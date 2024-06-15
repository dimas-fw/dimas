//! `DiMAS` observer example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("observer")
		.config(&Config::default())?;

	// create the observer for fibonacci
	agent
		.observer()
		.topic("fibonacci")
		.callback(|ctx, response| -> Result<()> { Ok(()) })
		.monitor(|ctx, feedback| -> Result<()> { Ok(()) })
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
