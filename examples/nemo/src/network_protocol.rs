//! Copyright © 2023 Stephan Kunz

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

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
	Alert(String),
	Warning(String),
	Info(String),
	Debug(String),
}
