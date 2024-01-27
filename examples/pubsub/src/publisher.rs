//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::time::Duration;
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

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// timer for regular publishing
	let duration = Duration::from_secs(1);
	let message = "Hello World!".to_string();
	let mut counter = 0i128;
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx, _props| {
			let text = message.clone() + " [" + &counter.to_string() + "]";
			println!("Sending '{}'", &text);
			// publishing with ad-hoc publisher
			let _ = ctx.publish("hello", text);
			counter += 1;
		})
		.add()?;

	// timer for regular deletion
	let duration = Duration::from_secs(3);
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx, _props| {
			println!("Deleting");
			// sending with ad-hoc delete
			let _ = ctx.delete("hello");
		})
		.add()?;

	agent.start().await;

	Ok(())
}
