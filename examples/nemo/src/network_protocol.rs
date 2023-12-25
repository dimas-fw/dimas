//! Copyright © 2023 Stephan Kunz

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr};

/// A type for representing data about a network node list.
#[derive(Debug, Default, Clone)]
pub struct NetworkDeviceList {
	pub nodes: HashMap<String, NetworkDevice>,
}

/// A type for representing data about a network node.
#[derive(Debug, Default, Clone)]
pub struct NetworkDevice {
	pub uuid: String,
	pub data: NetworkDeviceData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayIf {
	pub mac: String,
	pub address: IpAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfAddr {
	pub prefix_len: u8,
	pub address: IpAddr,
	pub broadcast: IpAddr,
	pub hostmask: IpAddr,
	pub netmask: IpAddr,
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
	pub gateway: Option<GatewayIf>,
	pub addresses: Vec<IfAddr>,
}

/// A type for representing data about a network tree.
#[derive(Debug, Default, Clone)]
pub struct NetworkTreeData {
	/// root of the tree.
	pub root: NetworkTreeNode,
	pub changed: bool,
}

/// A type for representing a node in a tree.
#[derive(Debug, Clone)]
pub enum NetworkTreeNode {
	Unknown {},
	Root { entries: Vec<NetworkTreeEntry> },
	Host { name: String, ip: String },
}

impl Default for NetworkTreeNode {
	fn default() -> Self {
		Self::Unknown {}
	}
}

/// A type for representing a tree entry.
#[derive(Debug, Default, Clone)]
pub struct NetworkTreeEntry {
	/// The name of this entry.
	pub name: String,
	/// The node for this entry.
	pub node: NetworkTreeNode,
}
