// Copyright © 2023 Stephan Kunz

// region:    --- modules
use std::time::SystemTime;
// endregion: --- modules

// region:    --- DimasMessage
pub struct DimasMessage<T> {
	timestamp: SystemTime,
	content: T,
}

impl<T> DimasMessage<T> {
	pub fn new(content: T) -> DimasMessage<T> {
		DimasMessage {
			timestamp: SystemTime::now(),
			content,
		}
	}

	pub fn content(&self) -> &T {
		&self.content
	}

	pub fn timestamp(&self) -> SystemTime {
		self.timestamp
	}
}
// endregion: --- DimasMessage
