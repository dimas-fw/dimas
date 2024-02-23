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
struct AgentProps {
	danube: Option<messages::StringMsg>,
	tagus: Option<messages::Pose>,
	missouri: Option<messages::Image>,
	godavari: Option<messages::LaserScan>,
	loire: Option<messages::PointCloud2>,
	yamuna: Option<messages::Vector3>,
	ohio: Option<messages::Float32>,
	volga: Option<messages::Float64>,
}

fn danube_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").danube = Some(value);
}

fn tagus_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Pose = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").tagus = Some(value);
}

fn missouri_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").missouri = Some(value);
}

fn brazos_callback(ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);

	let message = messages::Twist::random();
	let _ = ctx.put("congo", &message);
	info!("sent: '{}'", message);

	let message = messages::TwistWithCovarianceStamped::random();
	let _ = ctx.put("mekong", &message);
	info!("sent: '{}'", message);
}

fn yamuna_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Vector3 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").yamuna = Some(value);
}

fn godavari_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").godavari = Some(value);
}

fn loire_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").loire = Some(value);
}

fn ohio_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float32 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").ohio = Some(value);
}

fn volga_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Float64 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").volga = Some(value);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

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
