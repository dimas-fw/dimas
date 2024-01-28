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

#[derive(Debug)]
struct AgentProps {
	counter: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and the prefix given by `args`
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// use timer for regular publishing
	agent
		.timer()
		.interval(Duration::from_secs(1))
		.callback(|ctx, props| {
			let counter = props
				.read()
				.expect("should never happen")
				.counter
				.to_string();

			let text = "Hello World! [".to_string() + &counter + "]";
			println!("Sending '{}'", &text);
			// publishing with ad-hoc publisher
			let _ = ctx.publish("hello", text);
			props
				.write()
				.expect("should never happen")
				.counter += 1;
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
