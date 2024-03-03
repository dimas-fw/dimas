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

fn queryable(ctx: &ArcContext<AgentProps>, request: Request) -> Result<(), DimasError> {
	let value = ctx.read()?.counter;
	let query = request.key_expr();
	info!("Received query for {}, responding with {}", &query, &value);
	request.reply(value)?;

	ctx.write()?.counter += 1;
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new_with_prefix(Config::default(), properties, "examples");

	// add a queryable
	agent
		.queryable()
		.msg_type("query")
		.callback(queryable)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
