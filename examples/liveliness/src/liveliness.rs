//! `DiMAS` liveliness example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
// endregion:	--- modules

#[derive(Debug, Default)]
struct AgentProps {
	num: u32,
}

async fn put_callback(ctx: Context<AgentProps>, id: String) -> Result<()> {
	println!("{id} is alive");
	let mut val = ctx.read()?.num;
	val += 1;
	ctx.write()?.num = val;
	println!("Number of agents is {val}");
	Ok(())
}

async fn delete_callback(ctx: Context<AgentProps>, id: String) -> Result<()> {
	println!("{id} died");
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
		.put_callback(put_callback)
		.delete_callback(delete_callback)
		.add()?;

	// activate sending liveliness signal
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
