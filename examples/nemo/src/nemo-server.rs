//! The server/router for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

// region::    --- modules
use clap::Parser;
use dimas::prelude::*;
use zenoh::{config, prelude::SampleKind, sample::Sample};
//use nemo::network_protocol::*;
// endregion:: --- modules

// region::    --- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}
// endregion:: --- Clap

fn alert_subscription(sample: Sample) {
	//dbg!(&sample);
	match sample.kind {
		SampleKind::Put => {}
		SampleKind::Delete => {}
	}
}

fn liveliness_subscription(sample: Sample) {
	//dbg!(&sample);
	match sample.kind {
		SampleKind::Put => {}
		SampleKind::Delete => {}
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	let mut agent = Agent::new(config::peer(), &args.prefix);
	// activate sending liveliness
	agent.liveliness().await;

	// add a liveliness subscriber to listen for agents
	agent.liveliness_subscriber(liveliness_subscription).await;

	// listen for network alert messages
	agent
		.subscriber()
		.msg_type("alert")
		.callback(alert_subscription)
		.add()?;

	agent.start().await;
	Ok(())
}
