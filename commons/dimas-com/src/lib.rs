// Copyright © 2023 Stephan Kunz

//! Communicator implements the communication capabilities.
//!

// region:		--- modules
use dimas_core::error::{DimasError, Result};
use bitcode::{encode, Encode};
use std::fmt::Debug;
use std::sync::Arc;
use zenoh::prelude::{r#async::*, sync::SyncResolve};
use zenoh::publication::Publisher;
// endregion:	--- modules

// region:		--- Communicator
/// [`Communicator`] handles all communication aspects
#[derive(Debug)]
pub struct Communicator {
	/// The zenoh session
	pub session: Arc<Session>,
	/// A prefix to separate communication for different groups
	pub prefix: Option<String>,
}

impl Communicator {
	/// Constructor
	/// # Errors
	///
	pub fn new(config: dimas_config::Config) -> Result<Self> {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.map_err(DimasError::CreateSession)?,
		);
		Ok(Self {
			session,
			prefix: None,
		})
	}

	/// Get globally unique ID
	#[must_use]
	pub fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	/// Get group prefix
	#[must_use]
	pub const fn prefix(&self) -> &Option<String> {
		&self.prefix
	}

	/// Set group prefix
	pub fn set_prefix(&mut self, prefix: impl Into<String>) {
		self.prefix = Some(prefix.into());
	}

	/// Create a key expression from a topic by adding prefix if one is given.
	#[must_use]
	pub fn key_expr(&self, topic: &str) -> String {
		self.prefix
			.clone()
			.map_or_else(|| topic.into(), |prefix| format!("{prefix}/{topic}"))
	}

	/// Create a zenoh publisher
	/// # Errors
	///
	pub fn create_publisher<'a>(&self, key_expr: &str) -> Result<Publisher<'a>> {
		let p = self
			.session
			.declare_publisher(key_expr.to_owned())
			.res_sync()
			.map_err(DimasError::DeclarePublisher)?;
		Ok(p)
	}

	/// Send an ad hoc put `message` of type `M` using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	///
	#[allow(clippy::needless_pass_by_value)]
	pub fn put<M>(&self, topic: &str, message: M) -> Result<()>
	where
		M: Encode,
	{
		let value: Vec<u8> = encode(&message);
		let key_expr = self.key_expr(topic);

		self.session
			.put(&key_expr, value)
			.res_sync()
			.map_err(|_| DimasError::Put.into())
	}

	/// Send an ad hoc delete using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	///
	pub fn delete(&self, topic: &str) -> Result<()> {
		let key_expr = self.key_expr(topic);

		self.session
			.delete(&key_expr)
			.res_sync()
			.map_err(|_| DimasError::Delete.into())
	}
}
// endregion:	--- Communicator

#[cfg(test)]
mod tests {
	use super::*;
	//use serial_test::serial;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Communicator>();
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn communicator_create_multi() -> Result<()> {
		let mut peer1 = Communicator::new(dimas_config::Config::default())?;
		peer1.set_prefix("peer1");
		Ok(())
	}
}
