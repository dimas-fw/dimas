// Copyright © 2024 Stephan Kunz

//! The node 'tripoli' subscribes to
//! - an `Image` on the topic /columbia
//! - a `LaserScan` on the topic /godavari
//! and publishes on receive of /godavari a `PointCloud2` on topic /loire
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {
	columbia: Option<messages::Image>,
}

fn columbia_callback(
	_ctx: &Arc<Context<AgentProps>>,
	props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::Image = bitcode::decode(message).expect("should not happen");
	// just to see what has been sent
	info!("received: '{}'", &value);
	props.write().expect("should not happen").columbia = Some(value);
}

fn godavari_callback(
	ctx: &Arc<Context<AgentProps>>,
	_props: &Arc<RwLock<AgentProps>>,
	message: &Message,
) {
	let value: messages::LaserScan = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value);
	let msg = messages::PointCloud2::random();
	let _ = ctx.put_with("loire", &msg);
	info!("received: '{}'", msg);
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt().init();

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

	agent.publisher().msg_type("loire").add()?;

	agent.start().await;
	Ok(())
}
