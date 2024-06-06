//! `DiMAS` queryable example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

fn queryable(ctx: &Context<AgentProps>, request: Request) -> Result<()> {
	let received: u128 = request.decode()?;
	let value = ctx.read()?.counter;
	let query = request.key_expr();
	info!(
		"Received query for {} with {}, responding with {}",
		&query, &received, &value
	);
	request.reply(value)?;

	ctx.write()?.counter += 1;
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("queryable")
		.config(&Config::default())?;

	// add a queryable
	agent
		.queryable()
		.topic("query1")
		.callback(queryable)
		.add()?;

	// add a queryable
	agent
		.queryable()
		.topic("query2")
		.callback(queryable)
		.add()?;

	// add a queryable
	agent
		.queryable()
		.topic("query3")
		.callback(queryable)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
