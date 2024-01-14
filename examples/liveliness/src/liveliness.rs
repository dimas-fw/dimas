//! DiMAS liveliness example
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

fn liveliness_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let repl = ctx.prefix() + "/alive/";
	let agent_id = sample.key_expr.to_string().replace(&repl, "");
	match sample.kind {
		SampleKind::Put => {
			// born / started
			println!("{agent_id} is alive");
		}
		SampleKind::Delete => {
			// died / ended
			println!("{agent_id} is dead");
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

	// activate sending liveliness signal
	agent.liveliness(true);

	// add a liveliness subscriber to listen for other agents
	// the subscriber will also get its own liveliness signal
	agent
		.liveliness_subscriber()
		.callback(liveliness_subscription)
		.msg_type("alive")
		.add()
		.await?;

	agent.start().await;

	Ok(())
}
