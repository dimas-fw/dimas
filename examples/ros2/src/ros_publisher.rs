//! `DiMAS` ros2 publisher example
//! Copyright © 2024 Stephan Kunz

use std::time::Duration;

// region:		--- modules
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and no prefix
	let mut agent = Agent::new(properties).config(Config::default())?;

	// create publisher for topic "hello"
	agent.ros_publisher().topic("hello").add()?;

	// add a timer
	agent
		.timer()
		.name("ros2")
		.interval(Duration::from_secs(1))
		.callback(|ctx| -> Result<()> { 
			let counter = ctx.read()?.counter;

			let text = format!("Hello World! [{counter}]");
			info!("Sending '{}'", &text);
			// publishing with stored publisher
			//let _ = ctx.put_with("hello", text);
			Ok(()) 
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
