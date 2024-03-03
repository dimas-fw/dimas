//! `DiMAS` liveliness example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug, Default)]
struct AgentProps {
	test: u32,
}

fn liveliness_subscription(ctx: &ArcContext<AgentProps>, id: &str) -> Result<(), DimasError> {
	let _ = ctx.read()?.test;
	info!("{id} is alive");
	Ok(())
}

fn delete_subscription(ctx: &ArcContext<AgentProps>, id: &str) -> Result<(), DimasError> {
	let _ = ctx.read()?.test;
	info!("{id} died");
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps { test: 0 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new_with_prefix(Config::default(), properties, "examples")?;

	// add a liveliness subscriber to listen for other agents
	// the subscriber will also get its own liveliness signal
	agent
		.liveliness_subscriber()
		.put_callback(liveliness_subscription)
		.delete_callback(delete_subscription)
		.msg_type("alive")
		.add()?;

	// activate sending liveliness signal
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
