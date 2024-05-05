// Copyright © 2024 Stephan Kunz

//! Helper functions and structs
//!

// region:		--- modules
use crate::error::{DimasError, Result};
use dirs::{config_dir, config_local_dir, home_dir};
use std::env;
// endregion:	--- modules

// region:		--- functions
/// find a config file given by name
/// function will search in following directories for the file (order first to last):
///  - current working directory
///  - `.config` directory below current working directory
///  - `.config` directory below home directory
///  - local config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_LocalAppData}` | `MacOS`: `$HOME/Library/Application Support`)
///  - config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_RoamingAppData}` | `MacOS`: `$HOME/Library/Application Support`)
/// # Errors
///
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

		#[cfg(not(test))]
		let path = cwd.join(".config").join(filename);
		#[cfg(test)]
		let path = cwd.join("../.config").join(filename);
		if path.is_file() {
			return Ok(path);
		}
	};

	// handle typical config directories
	for path in [home_dir(), config_local_dir(), config_dir()]
		.into_iter()
		.flatten()
	{
		let file = path.join(filename);
		if file.is_file() {
			return Ok(path);
		}
	}

	Err(DimasError::FileNotFound(filename.into()).into())
}
// endregion:	--- functions

// region:    --- Tracing
/// Initialize tracing
pub fn init_tracing() {
	let subscriber = tracing_subscriber::fmt()
		//.with_env_filter(env_filter)
		.with_thread_ids(true)
		.with_thread_names(true)
		.with_level(true)
		.with_target(true);

	let subscriber = subscriber.finish();
	let _ = tracing::subscriber::set_global_default(subscriber);
}
// endregion: --- Tracing
