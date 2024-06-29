// Copyright © 2024 Stephan Kunz

//! Module `message_types` provides the different types of `Message`s used in callbacks.

// region:		--- modules
use crate::error::DimasError;
use bitcode::{decode, encode, Decode, Encode};
use std::ops::Deref;
use zenoh::{core::Wait, query::Query};
// endregion:	--- modules

// region:		--- Message
/// Implementation of a [`Message`].
#[derive(Debug)]
pub struct Message(pub Vec<u8>);

impl Deref for Message {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Message {
	/// Encode Message
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// Decode Message
	///
	/// # Errors
	pub fn decode<T>(self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		let value: Vec<u8> = self.0;
		decode::<T>(value.as_slice()).map_err(|_| DimasError::Decoding.into())
	}

	/// Get value of [`Message`]
	#[must_use]
	pub const fn value(&self) -> &Vec<u8> {
		&self.0
	}
}
// endregion:	--- Message

// region:    	--- QueryMsg
/// Implementation of a `Query` message handled by a `Queryable`
#[derive(Debug)]
pub struct QueryMsg(pub Query);

impl Deref for QueryMsg {
	type Target = Query;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl QueryMsg {
	/// Reply to the given [`QueryMsg`]
	///
	/// # Errors
	#[allow(clippy::needless_pass_by_value)]
	pub fn reply<T>(self, value: T) -> crate::error::Result<()>
	where
		T: Encode,
	{
		let key = self.0.selector().key_expr.to_string();
		let encoded: Vec<u8> = encode(&value);

		self.0
			.reply(&key, encoded)
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;
		Ok(())
	}

	/// Access the queries parameters
	#[must_use]
	pub fn parameters(&self) -> &str {
		self.0.parameters().as_str()
	}

	/// Decode [`QueryMsg`]
	///
	/// # Errors
	pub fn decode<T>(&self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		if let Some(value) = self.0.payload() {
			let content: Vec<u8> = value.into();
			return decode::<T>(content.as_slice()).map_err(|_| DimasError::Decoding.into());
		}
		Err(DimasError::NoMessage.into())
	}
}
// endregion: 	--- QueryMsg

// region:		--- QueryableMsg
/// Implementation of a `Queryable` message handled by a `Query`
#[derive(Debug)]
pub struct QueryableMsg(pub Vec<u8>);

impl Deref for QueryableMsg {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl QueryableMsg {
	/// Encode [`QueryableMsg`]
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// Decode [`QueryableMsg`]
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
// endregion:	--- QueryableMsg

// region:		--- ResponseType
#[derive(Debug, Encode, Decode)]
/// ?
pub enum ResponseType {
	/// ?
	Accepted(Vec<u8>),
	/// ?
	Declined,
}
// endregion:	--- ResponseType

// region:		--- ObserverMsg
/// Messages of an `Observer`
#[derive(Debug)]
pub struct ObserverMsg(pub Query);

impl Deref for ObserverMsg {
	type Target = Query;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ObserverMsg {
	/// Accept
	///
	/// # Errors
	pub fn accept(self) -> crate::error::Result<()> {
		let key = self.0.selector().key_expr.to_string();
		let content: Vec<u8> = Vec::new();
		let encoded: Vec<u8> = encode(&ResponseType::Accepted(content));

		self.0
			.reply(&key, encoded)
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;
		Ok(())
	}

	/// Decline
	///
	/// # Errors
	pub fn decline(self) -> crate::error::Result<()> {
		let key = self.0.selector().key_expr.to_string();
		let encoded: Vec<u8> = encode(&ResponseType::Declined);

		self.0
			.reply(&key, encoded)
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;
		Ok(())
	}

	/// Access the queries parameters
	#[must_use]
	pub fn parameters(&self) -> &str {
		self.0.parameters().as_str()
	}

	/// Decode [`ObserverMsg`]
	///
	/// # Errors
	pub fn decode<T>(&self) -> crate::error::Result<T>
	where
		T: for<'a> Decode<'a>,
	{
		if let Some(value) = self.0.payload() {
			let content: Vec<u8> = value.into();
			return decode::<T>(content.as_slice()).map_err(|_| DimasError::Decoding.into());
		}
		Err(DimasError::NoMessage.into())
	}
}
// endregion: 	--- ObserverMsg

// region:		--- ObservableMsg
/// Messages of an `Observable`
#[derive(Debug)]
pub struct ObservableMsg(pub Vec<u8>);

impl ObservableMsg {
	/// Encode Message
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// Decode [`ObservableMsg`]
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
// endregion: 	--- ObservableMsg

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Message>();
		is_normal::<QueryMsg>();
		is_normal::<QueryableMsg>();
		is_normal::<ObserverMsg>();
		is_normal::<ObservableMsg>();
	}
}