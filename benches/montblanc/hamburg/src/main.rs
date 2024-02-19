// Copyright © 2024 Stephan Kunz

//! The node 'hamburg' subscribes to
//!   - a Float32 on the topic /tigris
//!   - an Int64 on the topic /ganges
//!   - an Int32 on the topic /nile
//!   - a String on the topic /danube
//! and publishes the on /danube received value on topic /parana
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug, Default)]
struct AgentProps {
	ganges: i64,
	nile: i32,
	tigris: f32,
	danube: Option<String>,
}

fn tigris_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float32 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").tigris = value.data;
	println!("hamburg received: {:>14.6}", value.data);
}

fn ganges_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Int64 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").ganges = value.data;
	println!("hamburg received: {:>20}", value.data);
}

fn nile_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Int32 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").nile = value.data;
	println!("hamburg received: {:>12}", value.data);
}

fn danube_callback(ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	let _ = ctx.publish("parana", &value);
	println!("hamburg propagates: {}", &value.data);
	props.write().expect("should not happen").danube = Some(value.data);
}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps::default();
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
