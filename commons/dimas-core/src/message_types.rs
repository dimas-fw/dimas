// Copyright © 2024 Stephan Kunz

//! Module `message_types` provides the different types of `Message`s used in callbacks.

// region:		--- modules
use crate::error::DimasError;
use bitcode::{decode, encode, Decode, Encode};
use std::ops::Deref;
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
// endregion:	--- modules

// region:		--- Message
/// Implementation of a message received by subscriber callbacks
#[derive(Debug)]
pub struct Message(pub Vec<u8>);

impl Deref for Message {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Message {
	/// encode message
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// decode message
	///
	/// # Errors
	pub fn decode<T>(self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self.0;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}

	/// decode message
	#[must_use]
	pub const fn value(&self) -> &Vec<u8> {
		&self.0
	}
}
// endregion:	--- Message

// region:    --- Request
/// Implementation of a request for handling within a `Queryable`
#[derive(Debug)]
pub struct RequestMsg(pub Query);

impl Deref for RequestMsg {
	type Target = Query;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl RequestMsg {
	/// Reply to the given request
	///
	/// # Errors
	#[allow(clippy::needless_pass_by_value)]
	pub fn reply<T>(self, value: T) -> crate::error::Result<()>
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

	/// decode queries [`Message`]
	///
	/// # Errors
	pub fn decode<T>(&self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		if let Some(value) = self.0.value() {
			let content: Vec<u8> = value.try_into()?;
			return decode::<T>(content.as_slice()).map_err(|_| DimasError::Decoding.into());
		}
		Err(DimasError::NoMessage.into())
	}
}
// endregion: --- Request

// region:		--- Response
/// Implementation of a response received by query callbacks
#[derive(Debug)]
pub struct ResponseMsg(pub Vec<u8>);

impl Deref for ResponseMsg {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ResponseMsg {
	/// encode response
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// decode response
	///
	/// # Errors
	pub fn decode<T>(self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self.0;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Response

// region:		--- Feedback
/// Implementation of feedback messages
#[derive(Debug)]
pub struct FeedbackMsg(pub Vec<u8>);

impl Deref for FeedbackMsg {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl FeedbackMsg {
	/// encode response
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// decode feedback
	///
	/// # Errors
	///
	pub fn decode<T>(self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self.0;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Feedback

// region:		--- Result
/// Implementation of result messages
#[derive(Debug)]
pub struct ResultMsg(pub Vec<u8>);

impl Deref for ResultMsg {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ResultMsg {
	/// encode response
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// decode result
	///
	/// # Errors
	///
	pub fn decode<T>(self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self.0;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}
}
// endregion:	--- Result

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Message>();
		is_normal::<RequestMsg>();
		is_normal::<ResponseMsg>();
		is_normal::<FeedbackMsg>();
		is_normal::<ResultMsg>();
	}
}
