// Copyright © 2024 Stephan Kunz

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
//!  - local config directory (`Linux`: `$XDG_CONFIG_HOME/dimas` or `$HOME/.config/dimas` | `Windows`: `{FOLDERID_LocalAppData}/dimas` | `MacOS`: `$HOME/Library/Application Support/dimas`)
//!  - config directory (`Linux`: `$XDG_CONFIG_HOME/dimas` or `$HOME/.config/dimas` | `Windows`: `{FOLDERID_RoamingAppData}/dimas` | `MacOS`: `$HOME/Library/Application Support/dimas`)
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::utils::{find_config_file, read_config_file};
use crate::Result;
use alloc::vec::Vec;
use core::marker::PhantomData;
#[cfg(feature = "std")]
use tracing::{debug, warn};
// endregion:	--- modules

// region:		--- Session
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct Session {
	pub protocol: alloc::string::String,
	pub name: alloc::string::String,
	#[serde(deserialize_with = "zenoh::Config::deserialize")]
	pub config: zenoh::Config,
}

fn deserialize_sessions<'de, D>(deserializer: D) -> core::result::Result<Vec<Session>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	struct Sessions(PhantomData<Vec<Session>>);

	impl<'de> serde::de::Visitor<'de> for Sessions {
		type Value = Vec<Session>;

		fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
			formatter.write_str("string or list of strings")
		}

		//        fn visit_str<E>(self, value: &str) -> core::result::Result<Self::Value, E>
		//            where E: serde::de::Error
		//        {
		//			std::dbg!(value);
		//			std::panic!();
		//            Ok(std::vec![])
		//            //Ok(std::vec![value.to_owned()])
		//        }

		fn visit_seq<S>(self, visitor: S) -> core::result::Result<Self::Value, S::Error>
		where
			S: serde::de::SeqAccess<'de>,
		{
			serde::Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
		}
	}

	deserializer.deserialize_any(Sessions(PhantomData))
}
// endregion:	--- Session

// region:		--- Config
/// Manages the configuration
//#[repr(transparent)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
	#[serde(deserialize_with = "zenoh::Config::deserialize")]
	zenoh: zenoh::Config,
	#[serde(deserialize_with = "deserialize_sessions")]
	sessions: Vec<Session>,
}

#[cfg(not(feature = "std"))]
impl Default for Config {
	/// Create a default configuration
	fn default() -> Self {
		Self {
			zenoh: zenoh::Config::default(),
		}
	}
}

#[cfg(feature = "std")]
impl Default for Config {
	/// Create a default configuration
	///
	/// Will search for a configuration file with name "default.json5" in the directories mentioned in [`Examples`](index.html#examples).
	/// This file should contain the wanted default configuration.
	/// If no file is found, it will create a defined minimal default configuration.
	/// Currently this is just a default zenoh peer configuration which connects to peers in same subnet.
	#[allow(clippy::cognitive_complexity)]
	fn default() -> Self {
		match find_config_file("default.json5") {
			Ok(path) => {
				debug!("trying file {:?}", &path);
				match read_config_file(&path) {
					Ok(content) => match json5::from_str(&content) {
						Ok(result) => result,
						Err(error) => {
							warn!("{}, using default dimas configuration instead", error);
							Self {
								zenoh: zenoh::Config::default(),
								sessions: Vec::new(),
							}
						}
					},
					Err(error) => {
						warn!("{}, using default dimas configuration instead", error);
						Self {
							zenoh: zenoh::Config::default(),
							sessions: Vec::new(),
						}
					}
				}
			}
			Err(error) => {
				warn!("{}, using default dimas configuration instead", error);
				Self {
					zenoh: zenoh::Config::default(),
					sessions: Vec::new(),
				}
			}
		}
	}
}

impl Config {
	/// Create a configuration based on file named `local.json5`.
	///
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).
	/// This file should contain a configuration that only connects to entities on same host.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn local() -> Result<Self> {
		let path = find_config_file("local.json5")?;
		#[cfg(feature = "std")]
		debug!("using file {:?}", &path);
		let content = read_config_file(&path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `client.json5`.
	///
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).
	/// This file should contain a configuration that creates an entity in client mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn client() -> Result<Self> {
		let path = find_config_file("client.json5")?;
		debug!("using file {:?}", &path);
		let content = read_config_file(&path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `peer.json5`.
	///
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).
	/// This file should contain a configuration that creates an entity in peer mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn peer() -> Result<Self> {
		let path = find_config_file("peer.json5")?;
		debug!("using file {:?}", &path);
		let content = read_config_file(&path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file named `router.json5`.
	///
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).
	/// This file should contain a configuration that creates an entity in router mode.
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn router() -> Result<Self> {
		let path = find_config_file("router.json5")?;
		debug!("using file {:?}", &path);
		let content = read_config_file(&path)?;

		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Create a configuration based on file with given filename.
	///
	/// Will search in the directories mentioned in [`Examples`](index.html#examples).
	///
	/// # Errors
	/// Returns a [`std::io::Error`], if file does not exist in any of the places or is not accessible.
	#[cfg(feature = "std")]
	pub fn from_file(filename: &str) -> Result<Self> {
		let path = find_config_file(filename)?;
		debug!("using file {:?}", &path);
		let content = read_config_file(&path)?;
		let cfg = json5::from_str(&content)?;
		Ok(cfg)
	}

	/// Method to extract the zenoh configuration from [`Config`].
	///
	/// Can be passed to `zenoh::open()`.
	#[must_use]
	pub const fn zenoh_config(&self) -> &zenoh::Config {
		&self.zenoh
	}

	/// Method to get access to the the sessions in [`Config`].
	///
	#[must_use]
	pub const fn sessions(&self) -> &Vec<Session> {
		&self.sessions
	}
}
// endregion:	--- Config

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
