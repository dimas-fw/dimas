// Copyright © 2024 Stephan Kunz

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::{Error, Result};
use alloc::{
	boxed::Box,
	format,
	string::{String, ToString},
};
#[cfg(feature = "std")]
use dirs::{config_dir, config_local_dir, home_dir};
#[cfg(feature = "std")]
use std::env;
#[cfg(feature = "std")]
use std::path::PathBuf;
// endregion:	--- modules

// region:		--- utils
/// read a config file given by path
///
/// function parses for #include directives, loads the files and puts the content in place
/// # Errors
#[cfg(feature = "std")]
pub fn read_config_file(path: &PathBuf) -> Result<String> {
	let filename = path.to_string_lossy().to_string();
	let input = std::fs::read_to_string(path)?;
	parse_config_file(&filename, input)
}

/// parse a config string
///
/// function parses for #include directives, loads the files and puts the content in place
/// # Errors
#[cfg(feature = "std")]
pub fn parse_config_file(filename: &String, mut input: String) -> Result<String> {
	// we always start from the very beginning, thus we need no recursion to replace nested includes
	while let Some(pos) = input.find("#include ") {
		// calculate offsets
		let start = pos - 1;
		let len = input[pos..]
			.find('"')
			.ok_or_else(|| Error::InvalidInclude(filename.clone()))?;
		let end = 2 + start + len;
		// create file path to load
		let mut load_path = PathBuf::from(&filename);
		let load_file = input[start + 10..end - 1].to_string();
		// enhance relative path
		if load_file.starts_with('.') {
			let path = load_path
				.parent()
				.ok_or_else(|| Error::InvalidInclude(filename.clone()))?;
			load_path = path.join(load_file);
		}
		let content = std::fs::read_to_string(load_path)?;
		input.replace_range(start..end, &content);
	}
	Ok(input)
}

/// find a config file given by name
///
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
		let file = path.join("dimas").join(filename);
		if file.is_file() {
			return Ok(file);
		}
	}

	let text = format!("file {filename} not found");
	Err(Box::new(std::io::Error::new(
		std::io::ErrorKind::NotFound,
		text,
	)))
}
// endregion:	--- utils
