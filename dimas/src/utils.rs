// Copyright © 2024 Stephan Kunz

//! Helper functions and structs
//!

// region:		--- modules
use crate::{error::Result, prelude::DimasError};
use dirs::{config_dir, config_local_dir, home_dir};
use std::{
	env,
	sync::{mpsc::Receiver, Mutex},
	time::Duration,
};
// endregion:	--- modules

// region:		--- functions
/// find a config file given by name
/// function will search in following directories for the file (order first to last):
///  - current working directory
///  - `.config` directory below current working directory
///  - `.config` directory below home directory
///  - local config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_LocalAppData}` | `MacOS`: `$HOME/Library/Application Support`)
///  - config directory (`Linux`: `$XDG_CONFIG_HOME` or `$HOME/.config` | `Windows`: `{FOLDERID_RoamingAppData}` | `MacOS`: `$HOME/Library/Application Support`)
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

// region:		--- TaskSignal
#[derive(Debug, Clone)]
/// Internal signals, used by panic hooks to inform the [`Agent`] that someting has happened.
pub enum TaskSignal {
	/// Restart a certain liveliness subscriber, identified by its key expression.
	#[cfg(feature = "liveliness")]
	RestartLiveliness(String),
	/// Restart a certain queryable, identified by its key expression.
	#[cfg(feature = "queryable")]
	RestartQueryable(String),
	/// Restart a certain lsubscriber, identified by its key expression.
	#[cfg(feature = "subscriber")]
	RestartSubscriber(String),
	/// Restart a certain timer, identified by its key expression.
	#[cfg(feature = "timer")]
	RestartTimer(String),
	/// just to avoid warning messages when no feature is selected.
	#[allow(dead_code)]
	Dummy,
}

/// Wait non-blocking for [`TaskSignal`]s.<br>
/// Used by the `select!` macro within the [`Agent`]s main loop in [`Agent::start`].
pub async fn wait_for_task_signals(rx: &Mutex<Receiver<TaskSignal>>) -> Box<TaskSignal> {
	loop {
		if let Ok(signal) = rx.lock().expect("snh").try_recv() {
			return Box::new(signal);
		};
		// TODO: maybe there is a better solution than sleep
		tokio::time::sleep(Duration::from_millis(1)).await;
	}
}
// endregion:	--- TaskSignal

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
