// Copyright © 2024 Stephan Kunz

//! Module `message` provides the different types of `Message`s used in callbacks.

// region:		--- modules
use crate::{
	error::DimasError,
	prelude::{decode, encode, Decode, Encode},
};
use std::ops::Deref;
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
// endregion:	--- modules

// region:		--- Message
/// Implementation of a message received by subscriber callbacks
#[derive(Debug)]
pub struct Message {
	/// the key expression on which the message was sent
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

impl Message {
	/// decode message
	/// # Errors
	pub fn decode<T>(&self) -> Result<T, DimasError>
	where
		T: Decode,
	{
		decode::<T>(self).map_err(|_| DimasError::DecodingFailed)
	}
}
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
	/// # Errors
	///
	pub fn reply<T>(&self, value: T) -> Result<(), DimasError>
	where
		T: Encode,
	{
		let key = self.query.selector().key_expr.to_string();
		let encoded: Vec<u8> = encode(&value).map_err(|_| DimasError::ShouldNotHappen)?;
		let sample = Sample::try_from(key, encoded).map_err(|_| DimasError::ShouldNotHappen)?;

		self.query
			.reply(Ok(sample))
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;
		Ok(())
	}

	/// access the queries parameters
	#[must_use]
	pub fn parameters(&self) -> &str {
		self.query.parameters()
	}
}
// endregion: --- Request

// region:		--- Response
/// Implementation of a response received by query callbacks
#[derive(Debug)]
pub struct Response {
	/// the key expression for which the response was sent
	pub key_expr: String,
	/// the responses data
	pub value: Vec<u8>,
}

impl Deref for Response {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.value.as_slice()
	}
}

impl Response {
	/// decode response
	/// # Errors
	pub fn decode<T>(&self) -> Result<T, DimasError>
	where
		T: Decode,
	{
		decode::<T>(self).map_err(|_| DimasError::DecodingFailed)
	}
}
// endregion:	--- Response

