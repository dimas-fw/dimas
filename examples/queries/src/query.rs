//! `DiMAS` query example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use zenoh::{prelude::SampleKind, query::ConsolidationMode, sample::Sample};
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

fn query_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, sample: Sample) {
	// to avoid clippy message
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

	// timer for regular querying
	let duration = Duration::from_secs(1);
	let mut counter = 0i128;
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx, props| {
			println!("Querying [{counter}]");
			// querying with ad-hoc query
			ctx.query(
				ctx.clone(),
				props,
				"query",
				ConsolidationMode::None,
				query_callback,
			);
			counter += 1;
		})
		.add()?;

	agent.start().await;

	Ok(())
}
