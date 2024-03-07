// Copyright © 2024 Stephan Kunz

//! Module `config` provides `Config`, the configuration data for an `Agent`.

// region:		--- modules
use crate::error::DimasError;
use dirs::{config_dir, config_local_dir, home_dir};
use json::JsonValue;
use std::{env, path::PathBuf};
use tracing::{error, warn};
// endregion:	--- modules

// region:		--- utils
/// find a config file given by name
/// function will search in following directories for the file (order first to last):
///  - current directory
///  - .config directory below current directory
///  - .config directory below home directory
///  - local config directory (Linux: `$XDG_CONFIG_HOME` or $HOME/.config | `Windows: {FOLDERID_LocalAppData}` | `MacOS`: $HOME/Library/Application Support)
///  - config directory (Linux: `$XDG_CONFIG_HOME` or $HOME/.config | Windows: `{FOLDERID_RoamingAppData}` | `MacOS`: $HOME/Library/Application Support)
fn find_file(filename: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
	// handle environment path cwd
	if let Ok(cwd) = env::current_dir() {
		let path = cwd.join(filename);
		if path.is_file() {
			return read_file(path);
		}
		let path = cwd.join(".config").join(filename);
		if path.is_file() {
			return read_file(path);
		}
		#[cfg(test)]
		{
			let path = cwd.join("../.config").join(filename);
			dbg!(&path);
			if path.is_file() {
				return read_file(path);
			}	
		}
	};

	// handle typical config directories
	for path in [home_dir(), config_local_dir(), config_dir()]
		.into_iter()
		.flatten()
	{
		let file = path.join(filename);
		if file.is_file() {
			return read_file(file);
		}
	}

	Err(Box::new(DimasError::FileNotFound(filename.to_string())))
}

/// read a config file given by filepath
fn read_file(filepath: PathBuf) -> Result<JsonValue, Box<dyn std::error::Error>> {
	let text = std::fs::read_to_string(filepath)?;
	let json = json::parse(&text)?;
	Ok(json)
}

/// read a config file given by filepath
fn parse_content(content: &JsonValue) -> Result<Config, Box<dyn std::error::Error>> {
	let cfg = content
		.entries()
		.find(|item| item.0 == "zenoh")
		.map_or(Err(DimasError::NoZenohConfig), |result| {
			//dbg!(result.1.dump());
			match json5::Deserializer::from_str(&result.1.dump()) {
				Ok(mut d) => match zenoh::config::Config::from_deserializer(&mut d) {
					Ok(result) => Ok(result),
					Err(error) => match error {
						Ok(result) => Ok(result),
						Err(error) => Err(DimasError::ParseConfig(Box::new(error))),
					},
				},
				Err(error) => Err(DimasError::ParseConfig(Box::new(error))),
			}
		})?;
	Ok(Config(cfg))
}
// endregion:	--- utils

// region:		--- Config
/// Manages the agents configuration
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Config(zenoh::config::Config);

impl Default for Config {
	/// create a default configuration that connects to agents in same subnet
	#[allow(clippy::cognitive_complexity)]
	fn default() -> Self {
		match find_file("default.json5") {
			Ok(json) => match parse_content(&json) {
				Ok(result) => result,
				Err(error) => {
					error!("{}", error);
					warn!("using default zenoh peer configuration instead");
					Self(zenoh::config::peer())
				}
			},
			Err(error) => {
				error!("{}", error);
				warn!("using default zenoh peer configuration instead");
				Self(zenoh::config::peer())
			}
		}
	}
}

impl Config {
	/// create a configuration that only connects to agents on same host
	/// # Errors
	pub fn local() -> Result<Self, Box<dyn std::error::Error>> {
		let content = find_file("default.json5")?;
		let cfg = parse_content(&content)?;
		Ok(cfg)
	}

	/// create a low latency configuration
	/// # Errors
	pub fn low_latency() -> Result<Self, Box<dyn std::error::Error>> {
		let content = find_file("low_latency.json5")?;
		let cfg = parse_content(&content)?;
		Ok(cfg)
	}

	/// create a client configuration that connects to agents in same subnet
	/// # Errors
	pub fn client() -> Result<Self, Box<dyn std::error::Error>> {
		let content = find_file("client.json5")?;
		let cfg = parse_content(&content)?;
		Ok(cfg)
	}

	/// create a peer configuration that connects to agents in same subnet
	/// # Errors
	pub fn peer() -> Result<Self, Box<dyn std::error::Error>> {
		let content = find_file("peer.json5")?;
		let cfg = parse_content(&content)?;
		Ok(cfg)
	}

	/// create a router configuration
	/// # Errors
	pub fn router() -> Result<Self, Box<dyn std::error::Error>> {
		let content = find_file("router.json5")?;
		let cfg = parse_content(&content)?;
		Ok(cfg)
	}

	#[must_use]
	pub(crate) fn zenoh_config(&self) -> zenoh::config::Config {
		self.0.clone()
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
	fn default() {
		Config::default();
	}

	#[test]
	fn local() {
		Config::local().expect("");
	}

	#[test]
	fn router() {
		Config::router().expect("");
	}

	#[test]
	fn peer() {
		Config::peer().expect("");
	}

	#[test]
	fn client() {
		Config::client().expect("");
	}

	#[test]
	fn low_latency() {
		Config::low_latency().expect("");
	}
}
