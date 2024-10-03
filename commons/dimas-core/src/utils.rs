// Copyright © 2024 Stephan Kunz

//! Helper functions and structs
//!

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
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
pub fn selector_from(topic: &str, mut prefix: Option<&String>) -> String {
	prefix.take().map_or(
		topic.to_string(),
		|prefix| {
			let mut result = String::from(prefix);
			result.push('/');
			result.push_str(topic);
			result
		}
	)
}
// endregion: --- helper
