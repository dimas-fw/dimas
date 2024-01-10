//! DiMAS subscriber example
//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::{config, prelude::r#async::*};
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
	let _ = props;
	let _ = ctx;
	let message = serde_json::from_str::<String>(&sample.value.to_string())
		.unwrap()
		.to_owned();
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

	// create & initiaize agents properties
	let properties = AgentProps {};
	
  // create an agent with the properties
  let mut agent = Agent::new(config::peer(), &args.prefix, properties);
	
	// listen for 'hello' messages
	agent
		.subscriber()
		.msg_type("hello")
		.callback(hello_subscription)
		.add()
		.await?;

	agent.start().await;

	Ok(())
}
