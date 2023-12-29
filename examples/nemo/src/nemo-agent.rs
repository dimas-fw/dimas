//! The agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

// region::    --- modules
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
// endregion:: --- modules

// region::    --- Clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// prefix
	#[arg(short, long, value_parser, default_value_t = String::from("nemo"))]
	prefix: String,
}
// endregion:: --- Clap

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

	let mut agent = Agent::new(config::peer(), &args.prefix);

	// queryable for network interfaces
	agent
		.queryable()
		.msg_type("network")
		.callback(network)
		.add()?;

	// timer for volatile data with different interval
	let duration = Duration::from_secs(3);
	let sys = Arc::new(RwLock::new(System::new()));
	sys.write().unwrap().refresh_all();
	let sys_clone = sys.clone();
	agent
		.timer()
		.interval(duration)
		.callback(move |ctx| {
			sys_clone.write().unwrap().refresh_cpu();
			//dbg!(sys_clone.read().unwrap().global_cpu_info());
			//dbg!(&ctx);
			let message = NetworkMsg::Info("hi1".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()?;

	let duration = Duration::from_secs(10);
	agent
		.timer()
		.delay(duration)
		.interval(duration)
		.callback(move |ctx| {
			sys.write().unwrap().refresh_memory();
			//dbg!(sys.read().unwrap().free_memory());
			//dbg!(&ctx);
			let message = NetworkMsg::Alert("hi2".to_string());
			let _ = ctx.publish("alert", message);
		})
		.add()?;

	agent.start().await;

	Ok(())
}
