// Copyright © 2024 Stephan Kunz

//! Helper functions and structs
//!

// region:		--- modules
// endregion:	--- modules

// region:    --- tracing
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
// endregion: --- tracing

// region:    --- helper
/// create selector
#[must_use]
pub fn selector_from(topic: &str, prefix: Option<&String>) -> String {
	prefix.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"))
}
// endregion: --- helper
