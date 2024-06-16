//! `DiMAS` pingpong example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use chrono::Local;
use dimas::prelude::*;
use pingpong::PingPongMessage;
use std::time::Duration;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

#[allow(clippy::cast_precision_loss)]
fn pong_received(_ctx: &Context<AgentProps>, message: QueryableMsg) -> Result<()> {
	let message: PingPongMessage = message.decode()?;

	// get current timestamp
	let received = Local::now()
		.naive_utc()
		.and_utc()
		.timestamp_nanos_opt()
		.unwrap_or(0);
	// calculate & print traveltimes
	let oneway = received - message.received.unwrap_or(0);
	let roundtrip = received - message.sent;
	println!(
		"Trip {} from {} to {}, oneway {:.2}ms, roundtrip {:.2}ms",
		&message.counter,
		&message.ping_name,
		&message.pong_name,
		oneway as f64 / 1_000_000.0,
		roundtrip as f64 / 1_000_000.0
	);

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
		.name("ping")
		.config(&Config::default())?;

	// create query for topic "pingpong"
	agent
		.query()
		.topic("pingpong")
		.callback(pong_received)
		.add()?;

	// use timer for regular publishing
	agent
		.timer()
		.name("timer")
		.interval(Duration::from_secs(1))
		.callback(|ctx| -> Result<()> {
			let counter = ctx.read()?.counter;

			let message = PingPongMessage {
				counter,
				ping_name: hostname::get()?.into_string().unwrap_or(String::from("unknown host")),
				sent: Local::now()
					.naive_utc()
					.and_utc()
					.timestamp_nanos_opt()
					.unwrap_or(0),
				pong_name: String::from("unkown host"),
				received: None,
			};
			let message = Message::encode(&message);
			// publishing with stored publisher
			ctx.get("pingpong", Some(message), None)?;

			let text = format!("ping! [{counter}]");
			info!("Sent {} ", &text);

			// increase counter
			ctx.write()?.counter += 1;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
