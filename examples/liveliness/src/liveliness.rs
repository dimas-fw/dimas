//! `DiMAS` liveliness example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug, Default)]
struct AgentProps {
	num: u32,
}

fn liveliness_subscription(ctx: &ArcContext<AgentProps>, id: &str) -> Result<()> {
	info!("{id} is alive");
	let mut val = ctx.read()?.num;
	val += 1;
	ctx.write()?.num = val;
	println!("Number of agents is {val}");
	Ok(())
}

fn delete_subscription(ctx: &ArcContext<AgentProps>, id: &str) -> Result<()> {
	info!("{id} died");
	let mut val = ctx.read()?.num;
	val -= 1;
	ctx.write()?.num = val;
	println!("Number of agents is {val}");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps { num: 1 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("liveliness")
		.config(&Config::default())?;

	// add a liveliness subscriber to listen for other agents
	agent
		.liveliness_subscriber()
		.put_callback(liveliness_subscription)
		.delete_callback(delete_subscription)
		.add()?;

	// activate sending liveliness signal
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
