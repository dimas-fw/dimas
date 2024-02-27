// Copyright © 2024 Stephan Kunz

//use zenoh::config::ValidatedMap;

/// Manages the agents configuration
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Config(zenoh::config::Config);

impl Default for Config {
	/// create a default configuration that connecs to agents in same subnet
	fn default() -> Self {
		let res = zenoh::config::peer();
		// use every interface for connection
		//res.insert_json5("connect/endpoints", r#"["tcp/0.0.0.0/7447"]"#)
		//	.expect("could not create default config");
		Self(res)
	}
}

impl Config {
	/// create a configuration that only connects to agents on same host
	#[must_use]
	pub fn local() -> Self {
		let res = zenoh::config::peer();
		Self(res)
	}

	pub(crate) fn zenoh_config(&self) -> zenoh::config::Config {
		self.0.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Config>();
	}
}
