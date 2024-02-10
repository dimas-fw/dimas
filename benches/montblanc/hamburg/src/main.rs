// Copyright © 2024 Stephan Kunz

//! The node 'hanmurg' subscribes to 
//!   - a Float32 on the topic /tigris 
//!   - an Int64 on the topic /ganges 
//!   - an Int32 on the topic /nile 
//!   - a String on the topic /danube 
//! and publishes the on /danube received value on topic /parana
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug)]
struct AgentProps {}

fn tigris_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: f32 = bitcode::decode(message).expect("should not happen");
	println!("hamburg received: {value:>11.6}");
}

fn ganges_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: i64 = bitcode::decode(message).expect("should not happen");
	println!("hamburg received: {value:>20}");
}

fn nile_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: i32 = bitcode::decode(message).expect("should not happen");
	println!("hamburg received: {value:>14}");
}

fn danube_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: String = bitcode::decode(message).expect("should not happen");
	let _ = ctx.publish("parana", &value);
	println!("hamburg propagates: {value}");
}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(tigris_callback)
		.msg_type("tigris")
		.add()?;

		agent
		.subscriber()
		.put_callback(ganges_callback)
		.msg_type("ganges")
		.add()?;

		agent
		.subscriber()
		.put_callback(nile_callback)
		.msg_type("nile")
		.add()?;

		agent
		.subscriber()
		.put_callback(danube_callback)
		.msg_type("danube")
		.add()?;

	agent.start().await;
	Ok(())
}
