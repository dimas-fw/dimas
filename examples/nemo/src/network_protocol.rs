//! Copyright © 2023 Stephan Kunz

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use sysinfo::System;

/// Type for network UUID
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkUuid(pub String);

/// A type for representing a network device.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
	pub uuid: NetworkUuid,
	pub data: Option<NetworkDeviceData>,
	pub gateway: Option<NetworkUuid>,
}

/// A type for the data of a network device.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetworkDeviceData {
	//pub index: u32,
	pub up: bool,
	/// The name of this device.
	pub name: String,
	pub ifname: String,
	pub mac: String,
	pub addresses: Vec<IfAddr>,
}

/// A type for an interfaces set of addresses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfAddr {
	pub prefix_len: u8,
	pub address: IpAddr,
	pub broadcast: IpAddr,
	pub hostmask: IpAddr,
	pub netmask: IpAddr,
}

/// A type for network messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMsg {
	Fatal(String),
	Error(String),
	Warning(String),
	Hint(String),
	Info(String),
	Debug(String),
}

pub fn network_devices() -> Vec<NetworkDevice> {
	let mut res: Vec<NetworkDevice> = Vec::new();
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
				res.push(device);
			}
		}
	}
	res
}
