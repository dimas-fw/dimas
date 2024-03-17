//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	test: u8,
}

fn hello_publishing(_ctx: &ArcContext<AgentProps>, message: Message) -> Result<()> {
	let message: String = message.decode()?;
	info!("Received '{message}'");

	Ok(())
}

fn hello_deletion(ctx: &ArcContext<AgentProps>) -> Result<()> {
	let _value = ctx.read()?.test;
	info!("Shall delete 'hello' message");
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps { test: 0 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new_with_prefix(Config::default(), properties, "examples")?;

	// listen for 'hello' messages
	agent
		.subscriber()
		.topic("hello")
		.put_callback(hello_publishing)
		.delete_callback(hello_deletion)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
