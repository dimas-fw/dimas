// Copyright © 2024 Stephan Kunz

//! The node 'tripoli' subscribes to
//! - an `Image` on the topic /columbia
//! - a `LaserScan` on the topic /godavari
//! and publishes on receive of /godavari a `PointCloud2` on topic /loire
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {
	columbia: Option<messages::Image>,
}

fn columbia_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	// just to see what has been sent
	info!("received: '{}'", &value);
	props.write().expect("should not happen").columbia = Some(value);
}

fn godavari_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	let msg = messages::PointCloud2::random();
	let _ = ctx.publish("loire", &msg);
	info!("received: '{}'", msg);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::local(), properties);

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
