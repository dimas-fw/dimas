//! The agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use nemo::network_protocol::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use sysinfo::System;
use zenoh::{config, prelude::r#async::*, queryable::Query};
// endregion:	--- modules

// region:		--- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}
// endregion:	--- Clap

#[derive(Debug, Default)]
pub struct AgentProps {
	pub sys: System,
}

fn network(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, query: Query) {
	let _ = props;
	let _ = ctx;
	//dbg!("received");
	//dbg!(&query);
	tokio::spawn(async move {
		let key = query.selector().key_expr.to_string();
		let devices = network_devices();
		for device in devices {
			let sample =
				Sample::try_from(key.clone(), serde_json::to_string(&device).unwrap()).unwrap();
			//dbg!(&sample);
			query.reply(Ok(sample)).res().await.unwrap();
		}
	});
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create & initiaize agents properties
	let mut properties = AgentProps { sys: System::new() };
	properties.sys.refresh_all();
	let mut agent = Agent::new(config::peer(), &args.prefix, properties);
	// activate sending liveliness
	agent.liveliness(true);

	// queryable for network interfaces
	agent
		.queryable()
		.msg_type("network")
		.callback(network)
		.add()
		.await?;

	// timer for volatile data with different interval
	let duration = Duration::from_secs(3);
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx, props| {
			let mut props = props.write().unwrap();
			props.sys.refresh_cpu();
			//dbg!(sys_clone.read().unwrap().global_cpu_info());
			//dbg!(&ctx);
			//dbg!();
			let message = NetworkMsg::Info("hi1".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()
		.await?;

	let duration = Duration::from_secs(5);
	agent
		.timer()
		.delay(duration)
		.interval(duration)
		.callback(move |ctx, props| {
			let mut props = props.write().unwrap();
			props.sys.refresh_memory();
			//dbg!(sys.read().unwrap().free_memory());
			//dbg!(&ctx);
			//dbg!();
			let message = NetworkMsg::Hint("hi2".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()
		.await?;

	agent.start().await;

	Ok(())
}
