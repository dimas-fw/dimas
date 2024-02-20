// Copyright © 2024 Stephan Kunz

//! The node 'taipei' subscribes to an Image on the topic /columbia and publishes the received value on topic /colorado
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug)]
struct AgentProps {}

fn columbia_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let mut value: messages::Image = bitcode::decode(message).expect("should not happen");
	let height = value.height;
	let width = value.width;
	let id_old = value.header.frame_id.clone();
	value.header.frame_id = value.header.frame_id.replace("Test", "Modified");
	let id_new = value.header.frame_id.clone();
	let _ = ctx.publish("colorado", value);
	info!("taipei received: {height:>4} x {width:>4} -> {id_old}");
	info!("taipei sent: {height:>4} x {width:>4} -> {id_new}");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(columbia_callback)
		.msg_type("columbia")
		.add()?;

	agent.start().await;
	Ok(())
}
