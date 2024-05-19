//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use std::time::Duration;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	counter: u128,
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
		.name("publisher")
		.config(&Config::default())?;

	// create publisher for topic "hello"
	agent.publisher().topic("hello").add()?;

	// use timer for regular publishing
	agent
		.timer()
		.name("timer1")
		.interval(Duration::from_secs(1))
		.callback(|ctx| -> Result<()> {
			let counter = ctx.read()?.counter;

			let text = format!("Hello World! [{counter}]");
			info!("Sending '{}'", &text);
			// publishing with stored publisher
			let message = Message::encode(&text);
			let _ = ctx.put_with("hello", message);
			ctx.write()?.counter += 1;
			Ok(())
		})
		.add()?;

	// timer for regular deletion
	let duration = Duration::from_secs(3);
	agent
		.timer()
		.name("timer2")
		.interval(duration)
		.callback(move |ctx| -> Result<()> {
			info!("Deleting");
			// delete with stored publisher
			ctx.delete_with("hello")?;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
