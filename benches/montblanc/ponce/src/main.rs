// Copyright © 2024 Stephan Kunz

//! The node 'ponce' subscribes to
//!   - a `StringMsg` on the topic /danube
//!   - a `Pose` on the topic /tagus
//!   - an `Image` on the topic /missouri
//!   - a `PointCloud2` on the topic /brazos
//!   - a `Vector3` on the topic /yamuna
//!   - a `LaserScan` on the topic /godavari
//!   - a `PointCloud2` on the topic /loire
//!   - a `Float32` on the topic /ohio
//!   - a `Float64` on the topic /volga
//! and publishes on receive of tpoic /brazos
//!   - a `Twist` on the topic /congo
//!   - a `TwistWithCovarianceStampe` on the topic /mekong
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn danube_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("ponce received: {}", value.data);
}

fn tagus_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Pose = bitcode::decode(message).expect("should not happen");
	info!("ponce received Pose");
}

fn missouri_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Image = bitcode::decode(message).expect("should not happen");
	info!("ponce received Image");
}

fn brazos_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("ponce received PointCloud2");

	let message = messages::Twist::random();
	let _ = ctx.publish("congo", message);
	info!("mandalay sent Twist");

	let message = messages::TwistWithCovarianceStamped::random();
	let _ = ctx.publish("mekong", message);
	info!("mandalay sent TwistWithCovarianceStamped");
}

fn yamuna_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Vector3 = bitcode::decode(message).expect("should not happen");
	info!("ponce received Vector3");
}

fn godavari_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	info!("ponce received LaserScan");
}

fn loire_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("ponce received PointCloud2");
}

fn ohio_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float32 = bitcode::decode(message).expect("should not happen");
	info!("ponce received: {:>14.6}", value.data);
}

fn volga_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float64 = bitcode::decode(message).expect("should not happen");
	info!("ponce received: {:>17.6}", value.data);
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
		.put_callback(danube_callback)
		.msg_type("danube")
		.add()?;

	agent
		.subscriber()
		.put_callback(tagus_callback)
		.msg_type("tagus")
		.add()?;

	agent
		.subscriber()
		.put_callback(missouri_callback)
		.msg_type("missouri")
		.add()?;

	agent
		.subscriber()
		.put_callback(brazos_callback)
		.msg_type("brazos")
		.add()?;

	agent
		.subscriber()
		.put_callback(yamuna_callback)
		.msg_type("yamuna")
		.add()?;

	agent
		.subscriber()
		.put_callback(godavari_callback)
		.msg_type("godavari")
		.add()?;

	agent
		.subscriber()
		.put_callback(loire_callback)
		.msg_type("loire")
		.add()?;

	agent
		.subscriber()
		.put_callback(ohio_callback)
		.msg_type("ohio")
		.add()?;

	agent
		.subscriber()
		.put_callback(volga_callback)
		.msg_type("volga")
		.add()?;

	agent.start().await;
	Ok(())
}
