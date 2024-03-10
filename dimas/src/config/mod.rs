// Copyright © 2024 Stephan Kunz

//! Module `config` provides `Config`, the configuration data for an `Agent`.

// region:		--- modules
use crate::error::{DimasError, Result};
use dirs::{config_dir, config_local_dir, home_dir};
use std::env;
use tracing::{error, info, warn};
// endregion:	--- modules

// region:		--- utils
/// find a config file given by name
/// function will search in following directories for the file (order first to last):
///  - current directory
///  - .config directory below current directory
///  - .config directory below home directory
///  - local config directory (Linux: `$XDG_CONFIG_HOME` or $HOME/.config | `Windows: {FOLDERID_LocalAppData}` | `MacOS`: $HOME/Library/Application Support)
///  - config directory (Linux: `$XDG_CONFIG_HOME` or $HOME/.config | Windows: `{FOLDERID_RoamingAppData}` | `MacOS`: $HOME/Library/Application Support)
fn find_file(filename: &str) -> Result<String> {
	// handle environment path cwd
	if let Ok(cwd) = env::current_dir() {
		#[cfg(not(test))]
		let path = cwd.join(filename);
		#[cfg(test)]
		let path = cwd.join("..").join(filename);
		if path.is_file() {
			info!("using file {:?}", &path);
			return Ok(std::fs::read_to_string(path)?);
		}

		#[cfg(not(test))]
		let path = cwd.join(".config").join(filename);
		#[cfg(test)]
		let path = cwd.join("../.config").join(filename);
		if path.is_file() {
			info!("using file {:?}", &path);
			return Ok(std::fs::read_to_string(path)?);
		}
	};

	// handle typical config directories
	for path in [home_dir(), config_local_dir(), config_dir()]
		.into_iter()
		.flatten()
	{
		let file = path.join(filename);
		if file.is_file() {
			info!("using file {:?}", &path);
			return Ok(std::fs::read_to_string(path)?);
		}
	}

	Err(DimasError::FileNotFound(filename.to_string()).into())
}
// endregion:	--- utils

// region:		--- Config
/// Manages the agents configuration
#[repr(transparent)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
	#[serde(deserialize_with = "zenoh::config::Config::deserialize")]
	zenoh: zenoh::config::Config,
}

impl Default for Config {
	/// create a default configuration that connects to agents in same subnet
	#[allow(clippy::cognitive_complexity)]
	fn default() -> Self {
		match find_file("default.json5") {
			Ok(content) => match json5::from_str(&content) {
				Ok(result) => result,
				Err(error) => {
					error!("{}", error);
					warn!("using default zenoh peer configuration instead");
					Self {
						zenoh: zenoh::config::peer(),
					}
				}
			},
			Err(error) => {
				error!("{}", error);
				warn!("using default zenoh peer configuration instead");
				Self {
					zenoh: zenoh::config::peer(),
				}
			}
		}
	}
}

impl Config {
	/// create a configuration that only connects to agents on same host
	/// # Errors
	pub fn local() -> Result<Self> {
		let content = find_file("local.json5")?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// create a low latency configuration
	/// # Errors
	pub fn low_latency() -> Result<Self> {
		let content = find_file("low_latency.json5")?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// create a client configuration that connects to agents in same subnet
	/// # Errors
	pub fn client() -> Result<Self> {
		let content = find_file("client.json5")?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// create a peer configuration that connects to agents in same subnet
	/// # Errors
	pub fn peer() -> Result<Self> {
		let content = find_file("peer.json5")?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// create a router configuration
	/// # Errors
	pub fn router() -> Result<Self> {
		let content = find_file("router.json5")?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// create a configuration from a configuration file
	/// # Errors
	pub fn from_file(filename: &str) -> Result<Self> {
		let content = find_file(filename)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	#[must_use]
	pub(crate) fn zenoh_config(&self) -> zenoh::config::Config {
		self.zenoh.clone()
	}
}
// endregion:	--- Config

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Config>();
	}

	#[test]
	fn config_default() {
		Config::default();
	}

	#[test]
	fn config_local() {
		Config::local().expect("");
	}

	#[test]
	fn config_router() {
		Config::router().expect("");
	}

	#[test]
	fn config_peer() {
		Config::peer().expect("");
	}

	#[test]
	fn config_client() {
		Config::client().expect("");
	}

	#[test]
	fn config_low_latency() {
		Config::low_latency().expect("");
	}

	#[test]
	fn config_from_fle() {
		Config::from_file("default.json5").expect("");
	}

	#[test]
	#[should_panic = "non existent file"]
	fn config_from_fle_panics() {
		Config::from_file("non_existent.json5").expect("non existent file");
	}
}
