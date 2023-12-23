//! The host agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use clap::Parser;
use dimas::prelude::*;
use network_interface::*;
use std::time::Duration;
use sysinfo::{System, SystemExt};
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
        let sys = System::new();
        let key = query.selector().key_expr.to_string();
        let network_interfaces = NetworkInterface::show().unwrap();
        for itf in network_interfaces.iter() {
            //dbg!("{}\n", &itf);
            let mut device = NetworkDeviceData {
                name: sys.host_name().unwrap(),
                ifname: itf.name.clone(),
                index: itf.index,
                ..Default::default()
            };
            if itf.mac_addr.is_some() {
                device.mac = itf.mac_addr.as_ref().unwrap().clone();
            } else {
                device.mac = "".to_string();
            }
            let iter = itf.addr.iter();
            for addr in iter {
                // don't send loopback device infos
                if addr.ip().is_loopback() {
                    continue
                }
                let if_addr = IfAddr {
                    address: Some(addr.ip()),
                    broadcast: addr.broadcast(),
                    netmask: addr.netmask(),
                };
                device.addresses.push(if_addr);
            }
            if !device.addresses.is_empty() {
                //:g!("{}\n", &device);
                let sample =
                    Sample::try_from(key.clone(), serde_json::to_string(&device).unwrap()).unwrap();
                //dbg!(&sample);
                query.reply(Ok(sample)).res().await.unwrap();
            }
        }
    });
}

#[tokio::main]
async fn main() {
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

    // timer for volatile data
    let duration = Duration::from_secs(1);
    let mut sys = System::new();
    sys.refresh_system();
    agent.add_timer(Some(duration), Repetition::Interval(duration), move || {
        sys.refresh_cpu();
        sys.refresh_memory();
    });
    agent.start().await;
}
