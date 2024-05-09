// Copyright © 2024 Stephan Kunz

//! Module `message_types` provides the different types of `Message`s used in callbacks.

// region:		--- modules
use bitcode::{decode, encode, Decode, Encode};
use dimas_core::error::{DimasError, Result};
use std::ops::Deref;
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
// endregion:	--- modules

// region:		--- Message
/// Implementation of a message received by subscriber callbacks
#[derive(Debug)]
pub struct Message(pub Sample);

impl Deref for Message {
	type Target = Sample;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Message {
	/// decode message
	///
	/// # Errors
	pub fn decode<T>(self) -> Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self
			.0
			.value
			.try_into()
			.map_err(|_| DimasError::ConvertingValue)?;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Message

// region:    --- Request
/// Implementation of a request for handling within a `Queryable`
#[derive(Debug)]
pub struct Request(pub Query);

impl Deref for Request {
	type Target = Query;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Request {
	/// Reply to the given request
	///
	/// # Errors
	#[allow(clippy::needless_pass_by_value)]
	pub fn reply<T>(self, value: T) -> Result<()>
	where
		T: Encode,
	{
		let key = self.0.selector().key_expr.to_string();
		let encoded: Vec<u8> = encode(&value);
		let sample = Sample::try_from(key, encoded).map_err(|_| DimasError::ShouldNotHappen)?;

		self.0
			.reply(Ok(sample))
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;
		Ok(())
	}

	/// access the queries parameters
	#[must_use]
	pub fn parameters(&self) -> &str {
		self.0.parameters()
	}
}
// endregion: --- Request

// region:		--- Response
/// Implementation of a response received by query callbacks
#[derive(Debug)]
pub struct Response(pub Sample);

impl Deref for Response {
	type Target = Sample;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Response {
	/// decode response
	///
	/// # Errors
	pub fn decode<T>(self) -> Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self
			.0
			.value
			.try_into()
			.map_err(|_| DimasError::ConvertingValue)?;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Response

// region:		--- Feedback
/// Implementation of feedback messages
#[derive(Debug)]
pub struct Feedback(pub Sample);

impl Deref for Feedback {
	type Target = Sample;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Feedback {
	/// decode feedback
	///
	/// # Errors
	///
	#[allow(dead_code)]
	pub fn decode<T>(self) -> Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self
			.0
			.value
			.try_into()
			.map_err(|_| DimasError::ConvertingValue)?;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Feedback

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Message>();
		is_normal::<Request>();
		is_normal::<Response>();
		is_normal::<Feedback>();
	}
}
