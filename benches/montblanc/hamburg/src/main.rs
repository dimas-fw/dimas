// Copyright © 2024 Stephan Kunz

//! The node 'hamburg' subscribes to
//!   - a Float32 on the topic /tigris
//!   - an Int64 on the topic /ganges
//!   - an Int32 on the topic /nile
//!   - a String on the topic /danube
//! and publishes the on /danube received value on topic /parana
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {
	ganges: i64,
	nile: i32,
	tigris: f32,
}

fn tigris_callback(
	_ctx: &Arc<Context<AgentProps>>,
	props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::Float32 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").tigris = value.data;
	info!("received: '{}'", &value);
}

fn ganges_callback(
	_ctx: &Arc<Context<AgentProps>>,
	props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::Int64 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").ganges = value.data;
	info!("received: '{}'", &value);
}

fn nile_callback(
	_ctx: &Arc<Context<AgentProps>>,
	props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::Int32 = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").nile = value.data;
	info!("received: '{}'", &value);
}

fn danube_callback(
	ctx: &Arc<Context<AgentProps>>,
	_props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	let msg = messages::StringMsg {
		data: format!("hamburg/parana: {}", &value.data),
	};
	let _ = ctx.put_with("parana", &msg);
	info!("sent: '{msg}'");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent.publisher().msg_type("parana").add()?;

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
