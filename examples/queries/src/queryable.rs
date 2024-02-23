//! `DiMAS` queryable example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
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
struct AgentProps {
	counter: u128,
}

fn queryable(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, request: &Request) {
	let value = props
		.read()
		.expect("should never happen")
		.counter
		.to_string();
	info!("Received {}. query", &value);

	request.reply(&value);

	props
		.write()
		.expect("should never happen")
		.counter += 1;
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt().init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// add a queryable
	agent
		.queryable()
		.msg_type("query")
		.callback(queryable)
		.add()?;

	agent.start().await;

	Ok(())
}
