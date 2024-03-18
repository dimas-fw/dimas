// Copyright © 2024 Stephan Kunz

//! The [`Config`]uration data for an [`Agent`].
//!
//! # Examples
//! ```rust,no_run
//! # use dimas::prelude::*;
//! # main() {
//! // create a configuration from a file named `default.json5`
//! // located in one of the directories listed below
//! let config = Config::default();
//!
//! let config = Config::from_file("filename.sfx");    // use file named `filename.sfx`
//!
//! // a few more methods for standard filenames
//! // [example files](https://github.com/dimas-fw/dimas/tree/main/.config)
//! let config = Config::local();   // use file named `local.json5`
//! let config = Config::peer();    // use file named `peer.json5`
//! let config = Config::client();  // use file named `client.json5`
//! let config = Config::router();  // use file named `router.json5`
//! let config = Config::low_latency();  // use file named `low_latency.json5`
//!
//! // Configuration is handed over to the Agent
//! let agent = Agent::new(configuration, {});
//! # }
//! ```
//!
//! The methods using files will search in following directories for the file (order first to last):
//!  - current working directory
//!  - `.config` directory below current working directory
//!  - `.config` directory below home directory
//!  - local config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_LocalAppData}` | `MacOS`: `$HOME/Library/Application Support`)
//!  - config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_RoamingAppData}` | `MacOS`: `$HOME/Library/Application Support`)
//!

// region:		--- modules
#[allow(unused_imports)]
use crate::agent::Agent;
// endregion:	--- modules

// region:		--- modules
use crate::{error::Result, utils::find_config_file};
use tracing::{error, info, warn};
// endregion:	--- modules

// region:		--- utils
/// find and read a config file given by name
fn _read_file(filename: &str) -> Result<String> {
	// handle environment path current working directory `CWD`
	let path = find_config_file(filename)?;
	info!("using file {:?}", &path);
	Ok(std::fs::read_to_string(path)?)
}
// endregion:	--- utils

// region:		--- Config
/// Manages the [`Agent`]s configuration
#[repr(transparent)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
	#[serde(deserialize_with = "zenoh::config::Config::deserialize")]
	zenoh: zenoh::config::Config,
}

impl Default for Config {
	/// Create a default configuration<br>
	/// Will search for a configuration file with name "default.json5" in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain the wanted default configuration.<br>
	/// If no file is found, it will create a defined minimal default configuration.<br>
	/// Currently this is just a default zenoh peer configuration which connects to peers in same subnet.
	#[allow(clippy::cognitive_complexity)]
	fn default() -> Self {
		match find_config_file("default.json5") {
			Ok(path) => {
				info!("trying file {:?}", &path);
				match std::fs::read_to_string(path) {
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
	/// Create a configuration based on file named `local.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that only connects to [`Agent`]s on same host.
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn local() -> Result<Self> {
		let path = find_config_file("local.json5")?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `low_latency.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that only connects to [`Agent`]s on same host.
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn low_latency() -> Result<Self> {
		let path = find_config_file("low_latency.json5")?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `client.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an [`Agent`] in client mode.
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn client() -> Result<Self> {
		let path = find_config_file("client.json5")?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `peer.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an [`Agent`] in peer mode.
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn peer() -> Result<Self> {
		let path = find_config_file("peer.json5")?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `router.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an [`Agent`] in router mode.
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn router() -> Result<Self> {
		let path = find_config_file("router.json5")?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;

		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file with given filename.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	pub fn from_file(filename: &str) -> Result<Self> {
		let path = find_config_file(filename)?;
		info!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Internal method to reate a zenoh configuration from [`Config`].<br>
	/// Can be passed to `zenoh::open()`.
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
