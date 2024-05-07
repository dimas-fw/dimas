// Copyright © 2024 Stephan Kunz

//! Helper functions and structs
//!

// region:		--- modules
// endregion:	--- modules

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
