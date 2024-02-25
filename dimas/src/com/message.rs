// Copyright © 2024 Stephan Kunz

// region:		--- modules
use std::ops::Deref;
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
// endregion:	--- modules

// region:		--- Message
/// Implementation of a message received by subscriber callbacks
#[derive(Debug)]
pub struct Message {
	/// the kye expression on which the message was sent
	pub key_expr: String,
	/// the messages data
	pub value: Vec<u8>,
}

impl Deref for Message {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.value.as_slice()
	}
}

impl Message {}
// endregion:	--- Message

// region:    --- Request
/// Implementation of a request for handling within a `Queryable`
#[derive(Debug)]
pub struct Request {
	/// internal reference to zenoh `Query`
	pub(crate) query: Query,
}

impl Request {
	/// Reply to the given request
	/// # Panics
	///
	pub fn reply<T>(&self, value: T)
	where
		T: bitcode::Encode,
	{
		let key = self.query.selector().key_expr.to_string();
		let encoded: Vec<u8> = bitcode::encode(&value).expect("should never happen");
		let sample = Sample::try_from(key, encoded).expect("should never happen");

		self.query
			.reply(Ok(sample))
			.res_sync()
			.expect("should never happen");
	}

	/// access the queries parameters
	#[must_use]
	pub fn parameters(&self) -> &str {
		self.query.parameters()
	}
}
// endregion: --- Request
