//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use pubsub::PubSubMessage;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	count: u128,
}

async fn hello_publishing(ctx: Context<AgentProps>, message: Message) -> Result<()> {
	let message: PubSubMessage = message.decode()?;
	let count = ctx.read()?.count;
	if message.count != count {
		println!("missed {} messages", message.count - count);
		ctx.write()?.count = message.count;
	}
	println!("Received {} [{}]", message.text, message.count);
	ctx.write()?.count += 1;
	Ok(())
}

async fn hello_deletion(ctx: Context<AgentProps>) -> Result<()> {
	let _value = ctx.read()?.count;
	println!("Shall delete 'hello' message");
	Ok(())
}

#[tokio::main(worker_threads = 3)]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps { count: 0 };

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
