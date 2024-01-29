//! `DiMAS` liveliness example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
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

#[derive(Debug, Default)]
pub struct AgentProps {}

fn liveliness_subscription(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, agent_id: &str) {
	println!("{agent_id} is alive");
}

fn delete_subscription(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, agent_id: &str) {
	println!("{agent_id} died");
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// activate sending liveliness signal
	agent.liveliness(true);

	// add a liveliness subscriber to listen for other agents
	// the subscriber will also get its own liveliness signal
	agent
		.liveliness_subscriber()
		.put_callback(liveliness_subscription)
		.delete_callback(delete_subscription)
		.msg_type("alive")
		.add()?;

	agent.start().await;

	Ok(())
}
