// Copyright © 2024 Stephan Kunz

//! The node 'georgetown' subscribes to
//!   - a `WrenchStamped` on the topic /lena
//!   - a `Vector3Stamped` on the topic /murray
//! and publishes every 50ms
//!   - a `Float64` on the topic /volga
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn lena_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::WrenchStamped = bitcode::decode(message).expect("should not happen");
	info!("georgetown received WrenchStamped");
}

fn murray_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let _value: messages::Vector3Stamped = bitcode::decode(message).expect("should not happen");
	info!("georgetown received Vector3Stamped");
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
		.put_callback(lena_callback)
		.msg_type("lena")
		.add()?;

	agent
		.subscriber()
		.put_callback(murray_callback)
		.msg_type("murray")
		.add()?;

	agent
		.timer()
		.interval(Duration::from_millis(50))
		.callback(|ctx, _props| {
			let message = messages::Float64::random();
			let value = message.data;
			let _ = ctx.publish("volga", message);
			// just to see what value has been sent
			info!("georgetown sent: {value:>17.6}");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
