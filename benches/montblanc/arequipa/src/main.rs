// Copyright © 2024 Stephan Kunz

//! The node 'arequipa' subscribes to a `StringMsg` on the topic /arkansas
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Debug, Default)]
struct AgentProps {}

fn arkansas_callback(_ctx: &Arc<Context>, _props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("arequipa received: {}", &value.data);
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
		.put_callback(arkansas_callback)
		.msg_type("arkansas")
		.add()?;

	agent.start().await;
	Ok(())
}
