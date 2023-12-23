//! Copyright © 2023 Stephan Kunz

//use bincode::config;
//use dimas::prelude::Error;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr};
//use zenoh::{value::Value, buffers::{ZBuf, ZSlice}, prelude::{KnownEncoding, Encoding}};

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
    pub usage: Vec<NetworkTreeNode>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IfAddr {
    pub address: Option<IpAddr>,
    pub broadcast: Option<IpAddr>,
    pub netmask: Option<IpAddr>,
}

/// A type for the data of a network device.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetworkDeviceData {
    /// The name of this device.
    pub name: String,
    pub ifname: String,
    pub addresses: Vec<IfAddr>,
    pub mac: String,
    pub index: u32,
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
