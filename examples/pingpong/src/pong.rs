//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use chrono::Local;
use dimas::prelude::*;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

#[derive(Debug, Encode, Decode, Clone)]
struct PingPongMessage {
	counter: u128,
	sent: i64,
	received: Option<i64>,
}

fn ping_received(ctx: &ArcContext<AgentProps>, message: Message) -> Result<(), DimasError> {
	let mut message: PingPongMessage = message.decode()?;

	// set receive-timestamp
	message.received = Local::now().naive_utc().timestamp_nanos_opt();

	let text = "pong! [".to_string() + &message.counter.to_string() + "]";

	// publishing with ad-hoc publisher
	ctx.put_with("pong", message)?;

	info!("Sent '{}'", &text);

	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new_with_prefix(Config::default(), properties, "examples")?;

	// create publisher for topic "ping"
	agent.publisher().msg_type("pong").add()?;

	// listen for 'ping' messages
	agent
		.subscriber()
		.msg_type("ping")
		.put_callback(ping_received)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
