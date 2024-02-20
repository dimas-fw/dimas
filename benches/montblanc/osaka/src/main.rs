// Copyright © 2024 Stephan Kunz

//! The node 'osaka' subscribes to
//!   - a `String` on the topic /parana
//!   - an `Image` on the topic /columbia
//!   - an `Image` on the topic /colorado
//! and publishes on  receive of /colorado
//!   - a `PointCloud2` on the topic /salween
//!   - a `LaserScan` on the topic /godavari
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {
	parana: Option<String>,
	columbia: Option<messages::Image>,
	colorado: Option<messages::Image>,
}

fn parana_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	props.write().expect("should not happen").parana = Some(value.data.clone());
	info!("osaka received: {}", value.data);
}

fn columbia_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	let height = value.height;
	let width = value.width;
	let id = value.header.frame_id.clone();
	props.write().expect("should not happen").columbia = Some(value);
	info!("osaka received on /columbia: {height:>4} x {width:>4} -> {id}");
}

fn colorado_callback(ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	let height = value.height;
	let width = value.width;
	let id = value.header.frame_id.clone();
	props.write().expect("should not happen").colorado = Some(value);
	info!("osaka received on /colorado: {height:>4} x {width:>4} -> {id}");

	let message = messages::PointCloud2::random();
	let _ = ctx.publish("salween", &message);
	info!("osaka sentPointCloud2");

	let message = messages::LaserScan::random();
	let _ = ctx.publish("godavari", &message);
	info!("osaka sent LaserScan");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(parana_callback)
		.msg_type("parana")
		.add()?;

	agent
		.subscriber()
		.put_callback(columbia_callback)
		.msg_type("columbia")
		.add()?;

	agent
		.subscriber()
		.put_callback(colorado_callback)
		.msg_type("colorado")
		.add()?;

	agent.start().await;
	Ok(())
}
