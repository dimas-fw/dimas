// Copyright © 2024 Stephan Kunz
#![no_std]

//! The configuration data.
//!
//! An Agents configuration can be defined using json5 formated files.
//! There is a set of read methods for predefined filenames available.
//! You can find some example files [here](https://github.com/dimas-fw/dimas/tree/main/.config)
//!
//! # Examples
//! ```rust,no_run
//! # use dimas_config::Config;
//! # extern crate std;
//! # fn main() -> Result<(), Box<dyn core::error::Error + Send + Sync + 'static>> {
//! // in no-std environment:
//! // creates a default configuration
//! // in std environment:
//! // creates a configuration from a file named `default.json5`
//! // located in one of the directories listed below.
//! // If that file does not exist, a default config will be created
//! let config = Config::default();
//!
//! // use file named `filename.json5` (needs std environment)
//! // returns an error if file does not exist or is no valid configuration file
//! let config = Config::from_file("filename.json5")?;
//!
//! // methods with predefined filenames working like Config::from_file(...) (needs std environment)
//! let config = Config::local()?;        // use file named `local.json5`
//! let config = Config::peer()?;         // use file named `peer.json5`
//! let config = Config::client()?;       // use file named `client.json5`
//! let config = Config::router()?;       // use file named `router.json5`
//!
//! # Ok(())
//! # }
//! ```
//!
//! The methods using files will search in following directories for the file (order first to last):
//!  - current working directory
//!  - `.config` directory below current working directory
//!  - `.config` directory below home directory
//!  - local config directory (`Linux`: `$XDG_CONFIG_HOME/zenoh` or `$HOME/.config/zenoh` | `Windows`: `{FOLDERID_LocalAppData}/zenoh` | `MacOS`: `$HOME/Library/Application Support/zenoh`)
//!  - config directory (`Linux`: `$XDG_CONFIG_HOME/zenoh` or `$HOME/.config/zenoh` | `Windows`: `{FOLDERID_RoamingAppData}/zenoh` | `MacOS`: `$HOME/Library/Application Support/zenoh`)
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- types
/// copy of Result type alias from `dimas_core` to avoid dependency
pub(crate) type Result<T> =
	core::result::Result<T, Box<dyn core::error::Error + Send + Sync + 'static>>;
// endregion:	--- types

// region:		--- modules
use alloc::{boxed::Box, format, string::String};
#[cfg(feature = "std")]
use dirs::{config_dir, config_local_dir, home_dir};
#[cfg(feature = "std")]
use std::env;
#[cfg(feature = "std")]
use std::io::{Error, ErrorKind};
#[cfg(feature = "std")]
use tracing::{debug, warn};
// endregion:	--- modules

// region:		--- utils
/// find and read a config file given by name
#[cfg(feature = "std")]
fn _read_file(filename: &str) -> Result<String> {
	// handle environment path current working directory `CWD`
	let path = find_config_file(filename)?;
	debug!("using file {:?}", &path);
	Ok(std::fs::read_to_string(path)?)
}
// endregion:	--- utils

// region:		--- Config
/// Manages the configuration
#[repr(transparent)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
	#[serde(deserialize_with = "zenoh::Config::deserialize")]
	zenoh: zenoh::Config,
}

#[cfg(not(feature = "std"))]
impl Default for Config {
	/// Create a default configuration<br>
	fn default() -> Self {
		Self {
			zenoh: zenoh::Config::default(),
		}
	}
}

#[cfg(feature = "std")]
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
				debug!("trying file {:?}", &path);
				match std::fs::read_to_string(path) {
					Ok(content) => match json5::from_str(&content) {
						Ok(result) => result,
						Err(error) => {
							warn!("{}, using default zenoh configuration instead", error);
							Self {
								zenoh: zenoh::Config::default(),
							}
						}
					},
					Err(error) => {
						warn!("{}, using default zenoh configuration instead", error);
						Self {
							zenoh: zenoh::Config::default(),
						}
					}
				}
			}
			Err(error) => {
				warn!("{}, using default zenoh configuration instead", error);
				Self {
					zenoh: zenoh::Config::default(),
				}
			}
		}
	}
}

impl Config {
	/// Create a configuration based on file named `local.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that only connects to entities on same host.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn local() -> Result<Self> {
		let path = find_config_file("local.json5")?;
		#[cfg(feature = "std")]
		debug!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `client.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an entity in client mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn client() -> Result<Self> {
		let path = find_config_file("client.json5")?;
		debug!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `peer.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an entity in peer mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn peer() -> Result<Self> {
		let path = find_config_file("peer.json5")?;
		debug!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `router.json5`.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	/// This file should contain a configuration that creates an entity in router mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn router() -> Result<Self> {
		let path = find_config_file("router.json5")?;
		debug!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;

		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file with given filename.<br>
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).<br>
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn from_file(filename: &str) -> Result<Self> {
		let path = find_config_file(filename)?;
		debug!("using file {:?}", &path);
		let content = std::fs::read_to_string(path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Method to extract the zenoh configuration from [`Config`].<br>
	/// Can be passed to `zenoh::open()`.
	#[must_use]
	pub const fn zenoh_config(&self) -> &zenoh::Config {
		&self.zenoh
	}
}
// endregion:	--- Config

// region:		--- functions
/// find a config file given by name
/// function will search in following directories for the file (order first to last):
///  - current working directory
///  - `.config` directory below current working directory
///  - `.config` directory below home directory
///  - local config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_LocalAppData}` | `MacOS`: `$HOME/Library/Application Support`)
///  - config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_RoamingAppData}` | `MacOS`: `$HOME/Library/Application Support`)
/// # Errors
#[cfg(feature = "std")]
pub fn find_config_file(filename: &str) -> Result<std::path::PathBuf> {
	// handle environment path current working directory `CWD`

	if let Ok(cwd) = env::current_dir() {
		#[cfg(not(test))]
		let path = cwd.join(filename);
		#[cfg(test)]
		let path = cwd.join("..").join(filename);
		if path.is_file() {
			return Ok(path);
		}
		#[cfg(test)]
		let path = cwd.join("../..").join(filename);
		if path.is_file() {
			return Ok(path);
		}

		let path = cwd.join(".config").join(filename);
		if path.is_file() {
			return Ok(path);
		}
		#[cfg(test)]
		let path = cwd.join("../.config").join(filename);
		if path.is_file() {
			return Ok(path);
		}
		#[cfg(test)]
		let path = cwd.join("../../.config").join(filename);
		if path.is_file() {
			return Ok(path);
		}
	};

	// handle typical config directories
	for path in [home_dir(), config_local_dir(), config_dir()]
		.into_iter()
		.flatten()
	{
		let file = path.join("zenoh").join(filename);
		if file.is_file() {
			return Ok(file);
		}
	}

	let text = format!("file {filename} not found");
	Err(Box::new(Error::new(
		ErrorKind::NotFound,
		text,
	)))
}
// endregion:	--- functions

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Config>();
	}

	#[test]
	fn config_default() {
		Config::default();
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_local() -> Result<()> {
		Config::local()?;
		Ok(())
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_router() -> Result<()> {
		Config::router()?;
		Ok(())
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_peer() -> Result<()> {
		Config::peer()?;
		Ok(())
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_client() -> Result<()> {
		Config::client()?;
		Ok(())
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_from_file() -> Result<()> {
		Config::from_file("default.json5")?;
		Ok(())
	}

	#[cfg(feature = "std")]
	#[test]
	fn config_from_file_fails() {
		let _ = Config::from_file("non_existent.json5").is_err();
	}
}
