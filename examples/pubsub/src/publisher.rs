//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
use pubsub::PubSubMessage;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	count: u128,
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
			let count = ctx.read()?.count;
			// create structure to send
			let msg = PubSubMessage {
				count,
				text: String::from("hello world!"),
			};
			let message = Message::encode(&msg);
			println!("Sending {} [{}]", msg.text, msg.count);
			// publishing with stored publisher
			let _ = ctx.put("hello", message);
			ctx.write()?.count += 1;
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
			println!("Deleting");
			// delete with stored publisher
			ctx.delete("hello")?;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
