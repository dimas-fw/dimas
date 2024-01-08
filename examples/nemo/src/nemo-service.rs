//! The server/router for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use std::sync::{Arc, RwLock};

// region:		--- modules
use clap::Parser;
use dimas::prelude::*;
use nemo::network_protocol::*;
use zenoh::prelude::*;
//use nemo::network_protocol::*;
// endregion:	--- modules

// region:		--- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}
// endregion:	--- Clap

#[derive(Debug, Default)]
struct AgentProps {
	root: Option<Arc<NetworkTreeNode>>,
}

fn new_alert_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let _ = props;
	let _ = ctx;
	let _message = serde_json::from_str::<NetworkMsg>(&sample.value.to_string())
		.unwrap()
		.to_owned();
	//dbg!(&message);
	match sample.kind {
		SampleKind::Put => {
			//dbg!("as put");
		}
		SampleKind::Delete => {
			//dbg!("as delete");
		}
	}
}

fn liveliness_subscription(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let repl = ctx.prefix() + "/alive/";
	let agent_id = sample.key_expr.to_string().replace(&repl, "");
	let query_name = "network/".to_string() + &agent_id;
	//dbg!(&agent_id);
	//dbg!(&sample.key_expr);
	match sample.kind {
		SampleKind::Put => {
			// query remote device
			//dbg!("query");
			match ctx.query(
				ctx.clone(),
				props.clone(),
				query_name.clone(),
				ConsolidationMode::None,
				handle_query_response,
			) {
				Ok(_) => {}
				Err(error) => {
					dbg!(error);
					let _ = ctx.query(
						ctx.clone(),
						props.clone(),
						query_name,
						ConsolidationMode::None,
						handle_query_response,
					);
				}
			}
		}
		SampleKind::Delete => {
			remove_nodes(props.clone(), agent_id).unwrap();
		}
	}
}

fn handle_query_response(ctx: Arc<Context>, props: Arc<RwLock<AgentProps>>, sample: Sample) {
	let _ = ctx;
	//dbg!("response");
	//dbg!(&sample);
	let device = serde_json::from_str::<NetworkDevice>(&sample.value.to_string())
		.unwrap()
		.to_owned();
	//dbg!(&device);
	let repl = ctx.prefix() + "/network/";
	let agent_id = sample.key_expr.to_string().replace(&repl, "");
	//dbg!(&agent_id);
	add_node(props.clone(), agent_id, device).unwrap();
}

fn remove_nodes(props: Arc<RwLock<AgentProps>>, id: impl Into<String>) -> Result<()> {
	//dbg!("remove");
	// root comes from that agent
	let root = props.read().unwrap().root.clone();
	if root.is_some() {
		let agent_id = root.unwrap().clone().agent_id.clone();
		if agent_id == id.into() {
			let _x = props.write().unwrap().root.take();
		}
	}
	//dbg!(&props);
	Ok(())
}

fn add_node(
	props: Arc<RwLock<AgentProps>>,
	id: impl Into<String>,
	device: NetworkDevice,
) -> Result<()> {
	//dbg!("add");
	//dbg!(&props);
	let agent_id = id.into();
	//dbg!(&agent_id);
	// first have a look for the gateway in the tree
	if device.gateway.is_some() {
		let gw = device.gateway.unwrap().0;
		// empty root => gateway is new root and device is a child
		if props.read().unwrap().root.is_none() {
			let root = Arc::new(NetworkTreeNode {
				uuid: NetworkUuid(gw),
				agent_id: agent_id.clone(),
				data: None,
				gateway: None,
				children: RwLock::new(Vec::new()),
			});
			let child = Arc::new(NetworkTreeNode {
				uuid: device.uuid,
				agent_id: agent_id.clone(),
				data: device.data,
				gateway: Some(root.clone()), // danger of dbg!() loop
				children: RwLock::new(Vec::new()),
			});
			root.children.write().unwrap().push(child);
			props.write().unwrap().root.replace(root);
		}
	}

	//dbg!(&props);
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	let properties = AgentProps::default();
	let mut agent: Agent<AgentProps> = Agent::new(config::peer(), &args.prefix, properties);
	// de-activate sending liveliness
	// we rely on/need a nemo-agent running on the same machine
	agent.liveliness(false);

	// add a liveliness subscriber to listen for agents
	agent
		.liveliness_subscriber()
		.callback(liveliness_subscription)
		.msg_type("alive")
		.add()
		.await?;

	// listen for network alert messages
	agent
		.subscriber()
		.msg_type("alert")
		.callback(new_alert_subscription)
		.add()
		.await?;

	agent.start().await;
	Ok(())
}
