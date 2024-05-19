// Copyright © 2024 Stephan Kunz

// region:		--- modules
use std::{
	sync::{mpsc::Receiver, Mutex},
	time::Duration,
};
// endregion:	--- modules

// region:		--- TaskSignal
/// Internal signals, used by panic hooks to inform that someting has happened.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum TaskSignal {
	/// Restart a certain liveliness subscriber, identified by its key expression.
	RestartLiveliness(String),
	/// Restart a certain queryable, identified by its key expression.
	RestartQueryable(String),
	/// Restart a certain lsubscriber, identified by its key expression.
	RestartSubscriber(String),
	/// Restart a certain timer, identified by its key expression.
	RestartTimer(String),
}

/// Wait non-blocking for [`TaskSignal`]s.<br>
/// # Panics
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
