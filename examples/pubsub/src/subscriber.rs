//! `DiMAS` subscriber example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
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
	test: u8,
}

fn hello_publishing(_ctx: &ArcContext<AgentProps>, message: &Message) -> Result<(), DimasError> {
	let message: String = message.decode()?;
	info!("Received '{message}'");

	Ok(())
}

fn hello_deletion(ctx: &ArcContext<AgentProps>) -> Result<(), DimasError> {
	let _value = ctx.read()?.test;
	info!("Shall delete 'hello' message");
	Ok(())
}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { test: 0 };

	// create an agent with the properties
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// listen for 'hello' messages
	agent
		.subscriber()
		.msg_type("hello")
		.put_callback(hello_publishing)
		.delete_callback(hello_deletion)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await;

	Ok(())
}
