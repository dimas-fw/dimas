//! The server/router for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use std::sync::{Arc, RwLock};

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use zenoh::{config, prelude::SampleKind, sample::Sample};
//use nemo::network_protocol::*;
// endregion:	--- modules

// region:		--- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}
// endregion:	--- Clap

struct AgentProps {}

fn new_alert_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let _ = props;
	let _ = ctx;
	match sample.kind {
		SampleKind::Put => {
			dbg!(sample.value.to_string());
			//let message: M = serde_json::from_str(sample.value).unwrap().to_owned();
		}
		SampleKind::Delete => {
			todo!("received delete");
		}
	}
}

fn liveliness_subscription(sample: Sample) {
	//dbg!(&sample);
	match sample.kind {
		SampleKind::Put => {
			dbg!(&sample);
		}
		SampleKind::Delete => {
			dbg!(&sample);
		}
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	let properties = AgentProps {};
	let mut agent: Agent<AgentProps> = Agent::new(config::peer(), &args.prefix, properties);
	// activate sending liveliness
	agent.liveliness().await;

	// add a liveliness subscriber to listen for agents
	agent.liveliness_subscriber(liveliness_subscription).await;

	// listen for network alert messages
	agent
		.subscriber()
		.msg_type("alert")
		.callback(new_alert_subscription)
		.add()
		.await?;

	agent.start().await;
	Ok(())
}
