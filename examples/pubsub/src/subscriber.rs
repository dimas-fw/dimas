//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::{prelude::SampleKind, sample::Sample};
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

#[derive(Debug, Default)]
pub struct AgentProps {}

fn hello_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	// to avoid clippy message
	let _props = props;
	let _ctx = ctx;
	let sample = sample;
	let message =
		serde_json::from_str::<String>(&sample.value.to_string()).expect("could not deserialize");
	match sample.kind {
		SampleKind::Put => {
			println!("Received '{}'", &message);
		}
		SampleKind::Delete => {
			println!("Delete '{}'", &message);
		}
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// listen for 'hello' messages
	agent
		.subscriber()
		.msg_type("hello")
		.callback(hello_subscription)
		.add()
		.expect("should never happen");

	agent.start().await;

	Ok(())
}
