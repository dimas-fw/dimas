//! The agent for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use clap::Parser;
use dimas::prelude::*;
use std::{net::IpAddr, time::Duration};
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
                let mut device = NetworkDeviceData {
                    //index: interface.index,
                    up: interface.is_up(),
                    name: System::host_name().unwrap(),
                    ifname: interface.name,
                    mac: interface.mac_addr.unwrap().to_string(),
                    ..Default::default()
                };
                if let Some(name) = interface.friendly_name {
                    device.ifname = name;
                }
                if let Some(gateway) = interface.gateway {
                    let gw = GatewayIf {
                        mac: gateway.mac_addr.to_string(),
                        address: gateway.ip_addr,
                    };
                    device.gateway = Some(gw);
                }
                for addr in interface.ipv4 {
                    let if_addr = IfAddr {
                        prefix_len: addr.prefix_len,
                        address: IpAddr::from(addr.network()),
                        broadcast: IpAddr::from(addr.broadcast()),
                        hostmask: IpAddr::from(addr.hostmask()),
                        netmask: IpAddr::from(addr.netmask()),
                    };
                    device.addresses.push(if_addr);
                }
                for addr in interface.ipv6 {
                    let if_addr = IfAddr {
                        prefix_len: addr.prefix_len,
                        address: IpAddr::from(addr.network()),
                        broadcast: IpAddr::from(addr.broadcast()),
                        hostmask: IpAddr::from(addr.hostmask()),
                        netmask: IpAddr::from(addr.netmask()),
                    };
                    device.addresses.push(if_addr);
                }

                if !device.addresses.is_empty() {
                    //:g!("{}\n", &device);
                    let sample =
                        Sample::try_from(key.clone(), serde_json::to_string(&device).unwrap())
                            .unwrap();
                    //dbg!(&sample);
                    query.reply(Ok(sample)).res().await.unwrap();
                }
            }
        }

        //        let network_interfaces = NetworkInterface::show().unwrap();
        //        for itf in network_interfaces.iter() {
        //            //dbg!("{}\n", &itf);
        //            let mut device = NetworkDeviceData {
        //                name: System::host_name().unwrap(),
        //                ifname: itf.name.clone(),
        //                index: itf.index,
        //                ..Default::default()
        //            };
        //            if itf.mac_addr.is_some() {
        //                device.mac = itf.mac_addr.as_ref().unwrap().clone();
        //            } else {
        //                device.mac = "".to_string();
        //            }
        //            let iter = itf.addr.iter();
        //            for addr in iter {
        //                // don't send loopback device infos
        //                if addr.ip().is_loopback() {
        //                    continue
        //                }
        //                let if_addr = IfAddr {
        //                    address: Some(addr.ip()),
        //                    broadcast: addr.broadcast(),
        //                    netmask: addr.netmask(),
        //                    gateway: None,
        //                };
        //                device.addresses.push(if_addr);
        //            }
        //            if !device.addresses.is_empty() {
        //                //:g!("{}\n", &device);
        //                let sample =
        //                    Sample::try_from(key.clone(), serde_json::to_string(&device).unwrap()).unwrap();
        //                //dbg!(&sample);
        //                query.reply(Ok(sample)).res().await.unwrap();
        //            }
        //        }
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
    sys.refresh_all();
    agent.add_timer(Some(duration), Repetition::Interval(duration), move || {
        sys.refresh_cpu();
        sys.refresh_memory();
    });
    agent.start().await;
}
