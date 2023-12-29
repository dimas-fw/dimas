//! The agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

// region::    --- modules
use clap::Parser;
use dimas::prelude::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use sysinfo::System;
use zenoh::{config, prelude::r#async::*, queryable::Query};

use nemo::network_protocol::*;
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

fn network(query: Query) {
	//dbg!(&query);
	tokio::spawn(async move {
		let key = query.selector().key_expr.to_string();
		let interfaces = network_devices();
		for interface in interfaces {
			let sample =
				Sample::try_from(key.clone(), serde_json::to_string(&interface).unwrap()).unwrap();
			//dbg!(&sample);
			query.reply(Ok(sample)).res().await.unwrap();
		}
	});
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	let mut agent = Agent::new(config::peer(), &args.prefix);
	// activate sending liveliness
	agent.liveliness().await;

	// queryable for network interfaces
	agent
		.queryable()
		.msg_type("network")
		.callback(network)
		.add()?;

	// timer for volatile data with different interval
	let duration = Duration::from_secs(3);
	let sys = Arc::new(RwLock::new(System::new()));
	sys.write().unwrap().refresh_all();
	let sys_clone = sys.clone();
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx| {
			sys_clone.write().unwrap().refresh_cpu();
			//dbg!(sys_clone.read().unwrap().global_cpu_info());
			//dbg!(&ctx);
			let message = NetworkMsg::Info("hi1".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()?;

	let duration = Duration::from_secs(10);
	agent
		.timer()
		.delay(duration)
		.interval(duration)
		.callback(move |ctx| {
			sys.write().unwrap().refresh_memory();
			//dbg!(sys.read().unwrap().free_memory());
			//dbg!(&ctx);
			let message = NetworkMsg::Alert("hi2".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()?;

	agent.start().await;

	Ok(())
}
