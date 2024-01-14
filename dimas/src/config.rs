//! Copyright © 2024 Stephan Kunz

use zenoh::config::{ConnectConfig, ValidatedMap};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Config(zenoh::config::Config);

impl Default for Config {
	fn default() -> Self {
		let mut res = zenoh::config::peer();
		// use every interface for connection
		res.insert_json5("connect/endpoints", r#"["tcp/0.0.0.0/7447"]"#);
		Config(res)
	}
}

impl Config {
	pub fn local() -> Self {
		let res = zenoh::config::peer();
		Config(res)
	}

	pub(crate) fn zenoh_config(&self) -> zenoh::config::Config {
		self.0.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Config>();
	}
}