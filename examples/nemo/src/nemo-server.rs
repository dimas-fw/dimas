//! The server/router for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

// region::    --- modules
use clap::Parser;
use dimas::prelude::*;
use zenoh::{config, sample::Sample};
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

fn alert_subscription(_sample: Sample) {
	//dbg!(&sample);
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	let mut agent = Agent::new(config::peer(), &args.prefix);

	agent.liveliness();
	agent
		.subscriber()
		.msg_type("alert")
		.callback(alert_subscription)
		.add()?;

	agent.start().await;
	Ok(())
}
