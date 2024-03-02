//! `DiMAS` queryable example
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
	counter: u128,
}

fn queryable(ctx: &ArcContext<AgentProps>, request: Request) -> Result<(), DimasError> {
	let value = ctx.read()?.counter;
	let query = request.key_expr();
	info!("Received query for {}, responding with {}", &query,  &value);
	request.reply(value)?;

	ctx.write()?.counter += 1;
	Ok(())
}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(), DimasError> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt::init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// add a queryable
	agent
		.queryable()
		.msg_type("query")
		.callback(queryable)
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await;

	Ok(())
}
