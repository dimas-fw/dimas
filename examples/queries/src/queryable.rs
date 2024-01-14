//! DiMAS queryable example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::queryable::Query;
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

fn queryable(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, query: Query) {
	let _ = props;
	let _ = ctx;
	let key = query.selector().key_expr.to_string();
	dbg!(&query);
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// add a queryable
	agent
		.queryable()
		.msg_type("query")
		.callback(queryable)
		.add()
		.await?;

	agent.start().await;

	Ok(())
}
