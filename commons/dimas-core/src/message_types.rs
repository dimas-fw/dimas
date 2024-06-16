// Copyright © 2024 Stephan Kunz

//! Module `message_types` provides the different types of `Message`s used in callbacks.

// region:		--- modules
use crate::error::DimasError;
use bitcode::{decode, encode, Decode, Encode};
use std::ops::Deref;
use zenoh::{prelude::sync::SyncResolve, queryable::Query, sample::Sample};
// endregion:	--- modules

// region:		--- Message
/// Iimplementation of a Message.
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
	/// Reply to the given QueryMsg
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

	/// decode QueryMsg
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
	/// encode QueryableMsg
	pub fn encode<T>(message: &T) -> Self
	where
		T: Encode,
	{
		let content = encode(message);
		Self(content)
	}

	/// decode QueryableMsg
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

// region:		--- ObserverMsg
/// Messages of an Observer 
#[derive(Encode, Decode)]
pub enum ObserverMsg {
	/// Send a request to the Observable
	Request,
	/// Cancel request
	Cancel,
}

impl ObserverMsg {
	/// reply to an ObserverMsg
	#[allow(clippy::needless_pass_by_value)]
	pub fn reply<T>(self, value: T) -> crate::error::Result<()>
	where
		T: Encode,
	{
		Ok(())
	}
}
// endregion: 	--- ObserverMsg

// region:		--- ObservableMsg
/// Messages of an Observable
#[derive(Encode, Decode)]
pub enum ObservableMsg {
	/// Request was accepted
	Accepted,
	/// Request was declined
	Declined,
	/// Send the current status
	Status,
	/// Send successful end of request
	Finished,
	/// Send failure of request
	Failed,
	/// Acknowledge cancelation of request
	Canceled,
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
