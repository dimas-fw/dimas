//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use chrono::Local;
use dimas::prelude::*;
use std::time::Duration;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

#[derive(Debug, Encode, Decode)]
struct PingPongMessage {
	counter: u128,
	sent: i64,
	received: Option<i64>,
}

#[allow(clippy::cast_precision_loss)]
fn pong_received(_ctx: &ArcContext<AgentProps>, message: Message) -> Result<()> {
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
		"Trip {}, oneway {:.2}ms, roundtrip {:.2}ms",
		&message.counter,
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

	// create publisher for topic "ping"
	agent
		.publisher()
		.topic("ping")
		.set_priority(Priority::RealTime)
		.set_congestion_control(CongestionControl::Block)
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
				sent: Local::now()
					.naive_utc()
					.and_utc()
					.timestamp_nanos_opt()
					.unwrap_or(0),
				received: None,
			};

			// publishing with stored publisher
			ctx.put_with("ping", message)?;

			let text = format!("ping! [{counter}]");
			info!("Sent {} ", &text);

			// increase counter
			ctx.write()?.counter += 1;
			Ok(())
		})
		.add()?;

	// listen for 'pong' messages
	agent
		.subscriber()
		.topic("pong")
		.put_callback(pong_received)
		.set_reliability(Reliability::Reliable)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
