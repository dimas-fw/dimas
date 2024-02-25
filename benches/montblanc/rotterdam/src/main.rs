// Copyright © 2024 Stephan Kunz

//! The node 'rotterdam' subscribes to
//!   - a `TwistWithCovarianceStamped` on the topic /mekong
//! and publishes on receive
//!   - a `Vector3Stamped` on the topic /murray
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn mekong_callback(
	ctx: &Arc<Context<AgentProps>>,
	_props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::TwistWithCovarianceStamped =
		bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	let msg = messages::Vector3Stamped {
		header: value.header,
		vector: value.twist.twist.linear,
	};
	let _ = ctx.put_with("murray", &msg);
	info!("sent: '{}'", msg);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

	let properties = AgentProps::default();
	let mut agent = Agent::new(Config::default(), properties);

	agent.publisher().msg_type("murray").add()?;

	agent
		.subscriber()
		.put_callback(mekong_callback)
		.msg_type("mekong")
		.add()?;

	agent.start().await;
	Ok(())
}
