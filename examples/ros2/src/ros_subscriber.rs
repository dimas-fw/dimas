//! `DiMAS` ros2 subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

fn hello_callback(_ctx: &ArcContext<AgentProps>, message: Message) -> Result<()> {
	let message: String = message.decode()?;
	info!("Received '{message}'");

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and no prefix
	let mut agent = Agent::new(properties).config(Config::default())?;

		// listen for ROS2 'hello' messages
		agent
		.ros_subscriber()
		.topic("hello")
		.callback(hello_callback)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
