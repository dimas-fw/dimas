// Copyright © 2024 Stephan Kunz

// region:		--- modules
use std::{
	sync::{mpsc::Receiver, Mutex},
	time::Duration,
};
// endregion:	--- modules

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
/// # Panics
///
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
