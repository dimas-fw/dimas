//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use bitcode::{Decode, Encode};
use chrono::Local;
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;
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
	let mut message: PingPongMessage = bitcode::decode(message).expect("should not happen");

	// set receive-timestamp
	message.received = Local::now().naive_utc().timestamp_nanos_opt();

	let text = "pong! [".to_string() + &message.counter.to_string() + "]";

	// publishing with ad-hoc publisher
	let _ = ctx.put("pong", message);

	info!("Sent '{}'", &text);
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// listen for 'ping' messages
	agent
		.subscriber()
		.msg_type("ping")
		.put_callback(ping_received)
		.add()?;

	agent.start().await;

	Ok(())
}
