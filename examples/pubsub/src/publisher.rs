//! `DiMAS` publisher example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::time::Duration;
use tracing::info;
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

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties and the prefix given by `args`
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// create publisher for topic "hello"
	agent.publisher().msg_type("hello").add()?;

	// use timer for regular publishing
	agent
		.timer()
		.name("timer1")
		.interval(Duration::from_secs(1))
		.callback(|ctx| -> Result<(), DimasError> {
			let counter = ctx
				.read()?
				.counter
				.to_string();

			let text = "Hello World! [".to_string() + &counter + "]";
			info!("Sending '{}'", &text);
			// publishing with stored publisher
			let _ = ctx.put_with("hello", text);
			ctx.write()?.counter += 1;
			Ok(())
		})
		.add()?;

	// timer for regular deletion
	let duration = Duration::from_secs(3);
	agent
		.timer()
		.name("timer2")
		.interval(duration)
		.callback(move |ctx| -> Result<(), DimasError> {
			info!("Deleting");
			// delete with ad-hoc publisher
			ctx.delete("hello")?;
			Ok(())
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await?;

	Ok(())
}
