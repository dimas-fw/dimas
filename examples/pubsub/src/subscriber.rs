//! `DiMAS` subscriber example
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

fn hello_subscription(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let config = bincode::config::standard();
	let (message, _len): (String, usize) =
		bincode::decode_from_slice(message, config).expect("should not happen");
	println!("Received '{}'", &message);
}

fn hello_deletion(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>) {
	println!("Shall delete 'hello'");
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
		.put_callback(hello_subscription)
		.delete_callback(hello_deletion)
		.add()?;

	agent.start().await;

	Ok(())
}
