//! `DiMAS` query example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
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

#[derive(Debug, Default)]
struct AgentProps {}

fn query_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, answer: &[u8]) {
	let message: String = bitcode::decode(answer).expect("should not happen");
	info!("Received '{}'", &message);
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

	// timer for regular querying
	let duration = Duration::from_secs(1);
	let mut counter = 0i128;
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx, props| {
			info!("Querying [{counter}]");
			// querying with ad-hoc query
			ctx.query(ctx.clone(), props, "query", query_callback);
			counter += 1;
		})
		.add()?;

	agent.start().await;

	Ok(())
}
