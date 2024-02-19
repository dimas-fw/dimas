// Copyright © 2024 Stephan Kunz

//! The node 'tripoli' subscribes to
//! - an `Image` on the topic /columbia
//! - a `LaserScan` on the topic /godavari
//! and publishes on receive of /godavari a `PointCloud2` on topic /loire
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::sync::{Arc, RwLock};

use dimas::prelude::*;

#[derive(Debug)]
struct AgentProps {}

fn columbia_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	// just to see what has been sent
	println!(
		"tripoli received: {:>4} x {:>4} -> {}",
		value.height, value.width, value.header.frame_id
	);
}

fn godavari_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	let msg = messages::PointCloud2::random();
	let _ = ctx.publish("loire", msg);
	println!("tripoli received LaserScan");
	println!("tripoli sent PointCloud2");
}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps {};
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(columbia_callback)
		.msg_type("columbia")
		.add()?;

	agent
		.subscriber()
		.put_callback(godavari_callback)
		.msg_type("godavari")
		.add()?;

	agent.start().await;
	Ok(())
}
