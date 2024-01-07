//! The server/router for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use std::sync::{Arc, RwLock};

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use nemo::network_protocol::NetworkMsg;
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

#[derive(Debug)]
struct AgentProps {}

fn new_alert_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let _ = props;
	let _ = ctx;
	let message = serde_json::from_str::<NetworkMsg>(&sample.value.to_string()).unwrap().to_owned();
	dbg!(&message);
	match sample.kind {
		SampleKind::Put => {
			//dbg!("as put");
		}
		SampleKind::Delete => {
			//dbg!("as delete");
		}
	}
}

fn liveliness_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let _ = props;
	let _ = ctx;
	dbg!(&sample.key_expr);
	match sample.kind {
		SampleKind::Put => {
			//dbg!("as put");
		}
		SampleKind::Delete => {
			//dbg!("as delete");
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
	agent.liveliness("alive").await;

	// add a liveliness subscriber to listen for agents
	agent.liveliness_subscriber()
		.callback(liveliness_subscription)
		.msg_type("alive")
		.add().await?;

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
