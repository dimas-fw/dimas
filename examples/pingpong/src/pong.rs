//! `DiMAS` pong example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use chrono::Local;
use dimas::prelude::*;
use pingpong::PingPongMessage;
use tracing::info;
// endregion:	--- modules

#[derive(Debug)]
struct AgentProps {}

fn ping_received(_ctx: &Context<AgentProps>, message: QueryMsg) -> Result<()> {
	let mut query: PingPongMessage = message.decode()?;

	// set receive-timestamp
	query.received = Local::now()
		.naive_utc()
		.and_utc()
		.timestamp_nanos_opt();

	query.pong_name = hostname::get()?.into_string().unwrap_or(String::from("unknown host"));

	let text = format!("pong! [{}]", query.counter);

	// reply to ping query
	message.reply(query)?;

	info!("response '{}'", &text);

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// initialize tracing/logging
	init_tracing();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix 'examples'
	let mut agent = Agent::new(properties)
		.prefix("examples")
		.name("pong")
		.config(&Config::default())?;

	// listen for 'ping' messages
	agent
		.queryable()
		.topic("pingpong")
		.callback(ping_received)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
