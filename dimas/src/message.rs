// Copyright © 2023 Stephan Kunz

// region:		--- modules
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
// endregion:	--- modules

// region:		--- Message
pub trait Message {
	fn content<T>(&self) -> &T;
	fn utc(&self) -> NaiveDateTime;
}
// endregion:	--- Message

// region:		--- DimasMessage
#[derive(Serialize, Deserialize)]
pub struct DimasMessage<T> {
	utc: NaiveDateTime,
	content: T,
}

impl<T> DimasMessage<T> {
	pub fn new(content: T) -> DimasMessage<T> {
		DimasMessage {
			utc: Local::now().naive_utc(),
			content,
		}
	}
}

impl<T> Message for DimasMessage<T> {
	fn content<R>(&self) -> &R {
		todo!()
	}

	fn utc(&self) -> NaiveDateTime {
		self.utc
	}
}
// endregion:	--- DimasMessage

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	struct _Props {}

	#[test]
	fn normal_types() {
		//is_normal::<Message>();
		is_normal::<DimasMessage<_Props>>();
	}
}
