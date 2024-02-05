//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use bincode::{Decode, Encode};
use chrono::Local;
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
// endregion:	--- modules

// region:		--- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("examples"))]
	prefix: String,
}
// endregion:	--- Clap

#[derive(Debug)]
struct AgentProps {}

#[derive(Debug, Encode, Decode, Clone)]
struct PingPongMessage {
	counter: u128,
	sent: i64,
	received: Option<i64>,
}

fn ping_received(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let config = bincode::config::standard();
	let (mut message, _len): (PingPongMessage, usize) =
		bincode::decode_from_slice(message, config).expect("should not happen");

	// set receive-timestamp
	message.received = Local::now().naive_utc().timestamp_nanos_opt();

	// publishing with ad-hoc publisher
	let _ = ctx.publish("pong", message);

	let text = "pong!".to_string();
	println!("Sent '{}'", &text);
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// listen for 'ping' messages
	agent
		.subscriber()
		.msg_type("ping")
		.put_callback(ping_received)
		.add()?;

	agent.start().await;

	Ok(())
}
