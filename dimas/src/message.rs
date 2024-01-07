// Copyright © 2023 Stephan Kunz

// region:		--- modules
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::any::Any;
// endregion:	--- modules

// region:		--- DimasMessage
//pub trait DimasMessage {
//	type Msg;
//	fn content(&self) -> &Self::Msg;
//	fn utc(&self) -> NaiveDateTime;
//}
// endregion:	--- DimasMessage

// region:		--- Message
#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
	utc: NaiveDateTime,
	content: T,
}

impl<T> Message<T>
where
	T: Any,
{
	pub fn new(content: T) -> Message<T> {
		Message {
			utc: Local::now().naive_utc(),
			content,
		}
	}

	pub fn content(&self) -> &T {
		&self.content
	}

	pub fn utc(&self) -> NaiveDateTime {
		self.utc
	}
}

//impl<T> DimasMessage for Message<T> {
//	type Msg = T;
//	fn content(&self) -> &T {
//		todo!()
//	}
//
//	fn utc(&self) -> NaiveDateTime {
//		self.utc
//	}
//}
// endregion:	--- Message

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	struct _Props {}

	#[test]
	fn normal_types() {
		//is_normal::<DimasMessage>();
		is_normal::<Message<_Props>>();
	}
}
