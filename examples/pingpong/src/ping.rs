//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use bitcode::{Decode, Encode};
use chrono::Local;
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use std::time::Duration;
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
fn pong_received(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let message: PingPongMessage = bitcode::decode(message).expect("should not happen");

	// get current timestamp
	let received = Local::now()
		.naive_utc()
		.timestamp_nanos_opt()
		.unwrap_or(0);
	// calculate traveltimes
	let oneway = received - message.received.unwrap_or(0);
	let roundtrip = received - message.sent;
	info!(
		"Trip {}, oneway {:.2}ms, roundtrip {:.2}ms",
		&message.counter,
		oneway as f64 / 1_000_000.0,
		roundtrip as f64 / 1_000_000.0
	);
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt().init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and the prefix given by `args`
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// use timer for regular publishing
	agent
		.timer()
		.name("timer")
		.interval(Duration::from_secs(1))
		.callback(|ctx, props| {
			let counter = props.read().expect("should never happen").counter;

			let message = PingPongMessage {
				counter,
				sent: Local::now()
					.naive_utc()
					.timestamp_nanos_opt()
					.unwrap_or(0),
				received: None,
			};

			// publishing with ad-hoc publisher
			let _ = ctx.put("ping", message);

			let text = "ping! [".to_string() + &counter.to_string() + "]";
			info!("Sent {} ", &text);

			// increase counter
			props
				.write()
				.expect("should never happen")
				.counter += 1;
		})
		.add()?;

	// listen for 'pong' messages
	agent
		.subscriber()
		.msg_type("pong")
		.put_callback(pong_received)
		.add()?;

	agent.start().await;

	Ok(())
}
