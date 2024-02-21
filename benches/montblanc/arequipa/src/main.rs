// Copyright © 2024 Stephan Kunz

//! The node 'arequipa' subscribes to a `StringMsg` on the topic /arkansas and writes the data to a file
//!
//! This source is part of `DiMAS` implementation of Montblanc benchmark for distributed systems

use dimas::prelude::*;
use std::fs::File;
use std::{
	io::Write,
	sync::{Arc, RwLock},
};
use tracing::{error, info};

static OUT_FILE: &str = "/tmp/montblanc.out";

#[derive(Debug)]
struct AgentProps {
	file: File,
}

fn arkansas_callback(_ctx: &Arc<Context>, props: &Arc<RwLock<AgentProps>>, message: &[u8]) {
	let value: messages::StringMsg = bitcode::decode(message).expect("should not happen");
	info!("received: '{}'", &value.data);
	let final_data = format!("{}\n", value.data);
	props
		.write()
		.expect("should not happen")
		.file
		.write_all(final_data.as_bytes())
		.expect("should not happen");
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.init();

	let file = File::create(OUT_FILE).unwrap_or_else(|_| {
		error!("Could not create {OUT_FILE}");
		panic!("Could not create {OUT_FILE}");
	});
	let properties = AgentProps { file };
	let mut agent = Agent::new(Config::local(), properties);

	agent
		.subscriber()
		.put_callback(arkansas_callback)
		.msg_type("arkansas")
		.add()?;

	agent.start().await;
	Ok(())
}
