//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	test: u8,
}

async fn hello_publishing(_ctx: Context<AgentProps>, message: Message) -> Result<()> {
	let message: String = message.decode()?;
	println!("Received '{message}'");

	Ok(())
}

async fn hello_deletion(ctx: Context<AgentProps>) -> Result<()> {
	let _value = ctx.read()?.test;
	println!("Shall delete 'hello' message");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps { test: 0 };

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("subscriber")
		.config(&Config::default())?;

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
