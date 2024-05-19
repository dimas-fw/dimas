// Copyright © 2023 Stephan Kunz

//! Communicator implements the communication capabilities.
//!

// region:		--- modules
use dimas_core::{
	error::{DimasError, Result},
	message_types::{Message, Response},
};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use zenoh::prelude::{r#async::*, sync::SyncResolve};
// endregion:	--- modules

// region:		--- Communicator
/// [`Communicator`] handles all communication aspects
#[derive(Debug)]
pub struct Communicator {
	/// The zenoh session
	session: Arc<Session>,
	/// Mode of the session (router|peer|client)
	mode: String,
	/// A prefix to separate communication for different groups
	prefix: Option<String>,
}

impl Communicator {
	/// Constructor
	/// # Errors
	pub fn new(config: &dimas_config::Config) -> Result<Self> {
		let cfg = config.zenoh_config();
		let kind = cfg.mode().unwrap_or(WhatAmI::Peer).to_string();
		let session = Arc::new(
			zenoh::open(cfg)
				.res_sync()
				.map_err(DimasError::CreateSession)?,
		);
		Ok(Self {
			session,
			mode: kind,
			prefix: None,
		})
	}

	/// Get globally unique ID
	#[must_use]
	pub fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	/// Get session reference
	#[must_use]
	pub fn session(&self) -> Arc<Session> {
		self.session.clone()
	}

	/// Get session mode
	#[must_use]
	pub const fn mode(&self) -> &String {
		&self.mode
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

	/// Send an ad hoc put `message` of type `Message` using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	#[allow(clippy::needless_pass_by_value)]
	pub fn put(&self, topic: &str, message: Message) -> Result<()> {
		let key_expr = self.key_expr(topic);

		self.session
			.put(&key_expr, message.0)
			.res_sync()
			.map_err(|_| DimasError::Put.into())
	}

	/// Send an ad hoc delete using the given `topic`.
	/// The `topic` will be enhanced with the group prefix.
	/// # Errors
	pub fn delete(&self, topic: &str) -> Result<()> {
		let key_expr = self.key_expr(topic);

		self.session
			.delete(&key_expr)
			.res_sync()
			.map_err(|_| DimasError::Delete.into())
	}

	/// Send an ad hoc query using the given `selector`.
	/// Answers are collected via callback
	/// # Errors
	/// # Panics
	pub fn get<F>(&self, selector: &str, mut callback: F) -> Result<()>
	where
		F: FnMut(Response) + Sized,
	{
		let replies = self
			.session
			.get(selector)
			.consolidation(ConsolidationMode::None)
			.target(QueryTarget::All)
			.allowed_destination(Locality::Any)
			.timeout(Duration::from_millis(1000))
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let content: Vec<u8> = sample.value.try_into()?;
						callback(Response(content));
					}
					SampleKind::Delete => {
						println!("Delete in Query");
					}
				},
				Err(err) => {
					println!(
						">> Received (ERROR: '{}')",
						String::try_from(&err).expect("snh")
					);
				}
			}
		}
		Ok(())
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
		let cfg = dimas_config::Config::default();
		let mut peer = Communicator::new(&cfg)?;
		peer.set_prefix("test");
		Ok(())
	}
}
