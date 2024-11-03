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
use std::{
	env,
 	path::PathBuf,
	fs::File,
	io::{BufRead, BufReader}
};

// endregion:	--- modules

// region:		--- utils
/// read a config file given by path
///
/// function parses for #include directives, loads the files and puts the content in place
/// # Errors
#[cfg(feature = "std")]
pub fn read_config_file(path: &PathBuf) -> Result<String> {
	let filename = path.to_string_lossy().to_string();
	let input = read_file_without_comments(path)?;
	parse_config_file(&filename, input)
}

/// read file and remove comments
/// 
/// # Errors
#[cfg(feature = "std")]
fn read_file_without_comments(path: &PathBuf) -> Result<String> {
	let file = File::open(path)?;
    let reader = BufReader::new(file);
	let mut result = String::new();
    for line in reader.lines() {
		let line = line?.trim().to_string();
        let pos = line.find("//");
		if let Some(pos) = pos {
			result.push_str(&line[..pos]);
		} else {
			result.push_str(&line);
		}
    }

	Ok(result)
}

/// parse a config string
///
/// function parses for #include directives, loads the files and puts the content in place
/// # Errors
#[cfg(feature = "std")]
pub fn parse_config_file(filename: &String, mut input: String) -> Result<String> {
	// we always start from the very beginning
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
		let content = read_file_without_comments(&load_path)?;
		let content = parse_config_file(filename, content)?;
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
