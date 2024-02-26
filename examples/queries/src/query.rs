//! `DiMAS` query example
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
struct AgentProps {}

fn query_callback(
	_ctx: &Arc<Context<AgentProps>>,
	_props: &Arc<RwLock<AgentProps>>,
	response: &Message,
) {
	let message: String = decode(response).expect("should not happen");
	println!("Response '{}'", &message);
}

#[tokio::main]
async fn main() -> Result<()> {
	// a tracing subscriber writing logs
	tracing_subscriber::fmt().init();

	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new_with_prefix(Config::default(), properties, &args.prefix);

	// create publisher for topic "ping"
	agent
		.query()
		.msg_type("query")
		.callback(query_callback)
		.add()?;

	// timer for regular querying
	let duration = Duration::from_secs(1);
	let mut counter = 0i128;
	agent
		.timer()
		.name("timer")
		.interval(duration)
		.callback(move |ctx, _props| {
			info!("Querying [{counter}]");
			// querying with stored query
			ctx.get_with("query");
			counter += 1;
		})
		.add()?;

	// activate liveliness
	agent.liveliness(true);
	agent.start().await;

	Ok(())
}
