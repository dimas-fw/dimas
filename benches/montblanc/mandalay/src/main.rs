// Copyright © 2024 Stephan Kunz

//! The node 'mandalay' subscribes to
//!   - a `StringMsg` on the topic /danube
//!   - a `Quaternion` on the topic /chenab
//!   - a `PointCloud2` on the topic /salween
//!   - a `LaserScan` on the topic /godavari
//!   - a `Vector3` on the topic /yamuna
//!   - a `PointCloud2` on the topic /loire
//! and publishes every 100ms 
//!   - a `Pose` on the topic /tagus
//!   - an `Image` on the topic /missouri
//!   - a `PointCloud2` on the topic /brazos
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use std::{sync::{Arc, RwLock}, time::Duration};

use dimas::prelude::*;

#[derive(Debug, Default)]
struct AgentProps {}

fn danube_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	println!("mandalay received: {}", value.data);
}

fn chenab_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Quaternion = bitcode::decode(message).expect("should not happen");
	println!("mandalay received Quaternion");
}

fn salween_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	println!("mandalay received PointCloud2");
}

fn godavari_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	println!("mandalay received LaserScan");
}

fn yamuna_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Vector3 = bitcode::decode(message).expect("should not happen");
	println!("mandalay received Vector3");
}

fn loire_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	println!("mandalay received PointCloud2");
}

#[tokio::main]
async fn main() -> Result<()> {
	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent
		.subscriber()
		.put_callback(danube_callback)
		.msg_type("danube")
		.add()?;

	agent
		.subscriber()
		.put_callback(chenab_callback)
		.msg_type("chenab")
		.add()?;

	agent
		.subscriber()
		.put_callback(salween_callback)
		.msg_type("salween")
		.add()?;

	agent
		.subscriber()
		.put_callback(godavari_callback)
		.msg_type("godavari")
		.add()?;

		agent
		.subscriber()
		.put_callback(yamuna_callback)
		.msg_type("yamuna")
		.add()?;

		agent
		.subscriber()
		.put_callback(loire_callback)
		.msg_type("loire")
		.add()?;

		agent
		.timer()
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Pose::random();
			let _ = ctx.publish("tagus", message);
			println!("mandalay sent Pose");
		})
		.add()?;

		agent
		.timer()
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Image::random();
			let _ = ctx.publish("missouri", message);
			println!("mandalay sent Image");
		})
		.add()?;

		agent
		.timer()
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::PointCloud2::random();
			let _ = ctx.publish("brazos", message);
			println!("mandalay sent PointCloud2");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
