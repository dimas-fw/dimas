//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use serde::{Deserialize, Serialize};
use std::{
	net::IpAddr,
	sync::{Arc, RwLock},
};
use sysinfo::System;
// endregion:	---modules

// region:		--- types
/// Type for network UUID
#[repr(transparent)]
#[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkUuid(pub String);

impl std::fmt::Debug for NetworkUuid {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		//f.debug_tuple("NetworkUuid").field(&self.0).finish()
		f.write_str(&self.0)
	}
}
// endregion:	--- types

// region:		--- NetworkDevice
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
// endregion:	--- NetworkDevice

// region:		--- NetworkMsg
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
// endregion:	--- NetworkMsg

// region:		--- NetworkTree
/// A type for a tree of network devices
#[derive(Default)]
pub struct NetworkTreeNode {
	pub uuid: NetworkUuid,
	pub agent_id: String,
	pub data: Option<NetworkDeviceData>,
	pub gateway: Option<Arc<NetworkTreeNode>>,
	pub children: RwLock<Vec<Arc<NetworkTreeNode>>>,
}

impl NetworkTreeNode {}

impl std::fmt::Debug for NetworkTreeNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut gw = "None".to_string();
		if self.gateway.is_some() {
			gw = self.gateway.clone().unwrap().uuid.0.clone();
		}
		f.debug_struct("NetworkTreeNode")
			.field("uuid", &self.uuid)
			.field("agent_id", &self.agent_id)
			.field("data", &self.data)
			.field("gateway uuid", &gw)
			.field("children", &self.children.read().unwrap())
			.finish_non_exhaustive()
	}
}
// endregion:	--- NetworkTree

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<NetworkDevice>();
		is_normal::<NetworkDeviceData>();
		is_normal::<NetworkMsg>();
	}
}
