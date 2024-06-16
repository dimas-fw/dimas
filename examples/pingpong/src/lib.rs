//! `DiMAS` pingpong example
//! Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas::prelude::*;
// endregion:	--- modules

/// common structure for ping and pong
#[derive(Debug, Encode, Decode)]
pub struct PingPongMessage {
	/// counter
	pub counter: u128,
	/// ping's name
	pub ping_name: String,
	/// ping's sending timestamp
	pub sent: i64,
	/// pong's name
	pub pong_name: String,
	/// pong's receiving timestamp 
	pub received: Option<i64>,
}
