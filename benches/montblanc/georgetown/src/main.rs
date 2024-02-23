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
struct AgentProps {
	murray: Option<messages::Vector3Stamped>,
	lena: Option<messages::WrenchStamped>,
	volga: Option<messages::Float64>,
}

fn lena_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::WrenchStamped = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").lena = Some(value);
}

fn murray_callback(_ctx: &Arc<Context<AgentProps>>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::Vector3Stamped = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	props.write().expect("should not happen").murray = Some(value);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::local(), properties);

	agent.publisher().msg_type("volga").add()?;

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
		.name("timer")
		.interval(Duration::from_millis(50))
		.callback(|ctx, props| {
			let message = messages::Float64::random();
			let value = message.data;
			let _ = ctx.put_with("volga", &message);
			props.write().expect("should not happen").volga = Some(message);
			// just to see what value has been sent
			info!("sent: '{value}'");
		})
		.add()?;

	agent.start().await;
	Ok(())
}
