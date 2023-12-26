//! The agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use clap::Parser;
use dimas::prelude::*;
use std::{
	net::IpAddr,
	sync::{Arc, RwLock},
	time::Duration,
};
use sysinfo::System;
use zenoh::{config, prelude::r#async::*, queryable::Query};

use nemo::network_protocol::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}

fn network(query: Query) {
	//dbg!(&query);
	tokio::spawn(async move {
		let key = query.selector().key_expr.to_string();
		let interfaces = default_net::get_interfaces();
		for interface in interfaces {
			if !interface.is_loopback() {
				let mut data = NetworkDeviceData {
					//index: interface.index,
					up: interface.is_up(),
					name: System::host_name().unwrap(),
					ifname: interface.name,
					mac: interface.mac_addr.unwrap().to_string(),
					..Default::default()
				};
				if let Some(name) = interface.friendly_name {
					data.ifname = name;
				}
				for addr in interface.ipv4 {
					let if_addr = IfAddr {
						prefix_len: addr.prefix_len,
						address: IpAddr::from(addr.network()),
						broadcast: IpAddr::from(addr.broadcast()),
						hostmask: IpAddr::from(addr.hostmask()),
						netmask: IpAddr::from(addr.netmask()),
					};
					data.addresses.push(if_addr);
				}
				for addr in interface.ipv6 {
					let if_addr = IfAddr {
						prefix_len: addr.prefix_len,
						address: IpAddr::from(addr.network()),
						broadcast: IpAddr::from(addr.broadcast()),
						hostmask: IpAddr::from(addr.hostmask()),
						netmask: IpAddr::from(addr.netmask()),
					};
					data.addresses.push(if_addr);
				}

				if !data.addresses.is_empty() {
					//:g!("{}\n", &device);
					let uuid = NetworkUuid(interface.mac_addr.unwrap().to_string());
					let mut gateway = None;
					if let Some(gw) = interface.gateway {
						gateway = Some(NetworkUuid(gw.mac_addr.to_string()));
					}
					let data = Some(data);
					let device = NetworkDevice {
						uuid,
						data,
						gateway,
					};
					let sample =
						Sample::try_from(key.clone(), serde_json::to_string(&device).unwrap())
							.unwrap();
					//dbg!(&sample);
					query.reply(Ok(sample)).res().await.unwrap();
				}
			}
		}
	});
}

#[tokio::main]
async fn main() -> Result<()> {
	// parse arguments
	let args = Args::parse();

	// create agent
	let mut agent = Agent::new(config::peer(), &args.prefix);
	let agent_id = agent.uuid();
	//dbg!(&agent_uid);

	// queryable for network interfaces
	let network_query = agent_id + "/network";
	//dbg!(&network_query);
	agent.add_queryable(&network_query, network).await;

	// timer for volatile data with different interval
	let duration = Duration::from_secs(1);
	let sys = Arc::new(RwLock::new(System::new()));
	sys.write().unwrap().refresh_all();
	let sys_clone = sys.clone();
	agent.add_timer(Some(duration), Repetition::Interval(duration), move || {
		sys_clone.write().unwrap().refresh_cpu();
		//dbg!(sys_clone.read().unwrap().global_cpu_info());
	});
	let duration = Duration::from_secs(10);
	agent.add_timer(None, Repetition::Interval(duration), move || {
		sys.write().unwrap().refresh_memory();
		//dbg!(sys.read().unwrap().free_memory());
	});
	agent.start().await;

	Ok(())
}
