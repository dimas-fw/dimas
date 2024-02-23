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

use dimas::prelude::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {
	danube: Option<messages::StringMsg>,
	chenab: Option<messages::Quaternion>,
	salween: Option<messages::PointCloud2>,
	godavari: Option<messages::LaserScan>,
	loire: Option<messages::PointCloud2>,
	yamuna: Option<messages::Vector3>,
}

fn danube_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").danube = Some(value);
}

fn chenab_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Quaternion = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").chenab = Some(value);
}

fn salween_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").salween = Some(value);
}

fn godavari_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").godavari = Some(value);
}

fn yamuna_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Vector3 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").yamuna = Some(value);
}

fn loire_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::PointCloud2 = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").loire = Some(value);
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

	agent.publisher().msg_type("tagus").add()?;

	agent
		.timer()
		.name("timer")
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Pose::random();
			let _ = ctx.put_with("tagus", &message);
			info!("sent: '{}'", message);
		})
		.add()?;

	agent
		.timer()
		.name("timer")
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::Image::random();
			let _ = ctx.put_with("missouri", message);
			info!("mandalay sent Image");
		})
		.add()?;

	agent
		.timer()
		.name("timer")
		.interval(Duration::from_millis(100))
		.callback(|ctx, _props| {
			let message = messages::PointCloud2::random();
			let _ = ctx.put_with("brazos", message);
			info!("mandalay sent PointCloud2");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
