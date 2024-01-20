//! `DiMAS` queryable example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
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
pub struct AgentProps {
	counter: i128,
}

fn queryable(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, query: Query) {
	//dbg!(&query);
	// to avoid clippy message
	let _ctx = ctx;
	let p = props;
	let query = query;
	let value = p.read().expect("should never happen").counter.to_string();
	println!("Received query {}", &value);

	let key = query.selector().key_expr.to_string();
	let sample = Sample::try_from(
		key,
		serde_json::to_string(&value)
			.expect("should never happen"),
	)
	.expect("should never happen");

	query
		.reply(Ok(sample))
		.res_sync()
		.expect("should never happen");

	p.write().expect("should never happen").counter += 1;
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initialize agents properties
	let properties = AgentProps { counter: 0 };

	// create an agent with the properties
	let mut agent = Agent::new(Config::default(), &args.prefix, properties);

	// add a queryable
	agent
		.queryable()
		.msg_type("query")
		.callback(queryable)
		.add()?;

	agent.start().await;

	Ok(())
}
