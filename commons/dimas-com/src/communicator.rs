// Copyright © 2023 Stephan Kunz

//! Communicator implements the communication capabilities.
//!

// region:		--- modules
use core::fmt::Debug;
use dimas_core::{
	error::{DimasError, Result},
	message_types::{Message, QueryableMsg},
};
use std::sync::Arc;
#[cfg(feature = "unstable")]
use zenoh::config::WhatAmI;
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	query::{ConsolidationMode, QueryTarget},
	sample::SampleKind,
	Session, Wait,
};
// endregion:	--- modules

// region:		--- Communicator
/// [`Communicator`] handles all communication aspects
#[derive(Debug)]
pub struct Communicator {
	/// The zenoh session
	session: Arc<Session>,
	/// Mode of the session (router|peer|client)
	mode: String,
}

impl Communicator {
	/// Constructor
	/// # Errors
	pub fn new(config: &dimas_config::Config) -> Result<Self> {
		let cfg = config.zenoh_config();
		#[cfg(feature = "unstable")]
		let kind = cfg.mode().unwrap_or(WhatAmI::Peer).to_string();
		#[cfg(not(feature = "unstable"))]
		let kind = String::from("unknown");
		let session = Arc::new(
			zenoh::open(cfg)
				.wait()
				.map_err(DimasError::CreateSession)?,
		);
		Ok(Self {
			session,
			mode: kind,
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

	/// Send an ad hoc put `message` of type `Message` using the given `selector`.
	/// # Errors
	#[allow(clippy::needless_pass_by_value)]
	pub fn put(&self, selector: &str, message: Message) -> Result<()> {
		self.session
			.put(selector, message.value())
			.wait()
			.map_err(|_| DimasError::Put.into())
	}

	/// Send an ad hoc delete using the given `selector`.
	/// # Errors
	pub fn delete(&self, selector: &str) -> Result<()> {
		self.session
			.delete(selector)
			.wait()
			.map_err(|_| DimasError::Delete.into())
	}

	/// Send an ad hoc query with an optional [`Message`] using the given `selector`.
	/// Answers are collected via callback
	/// # Errors
	/// # Panics
	pub fn get<F>(&self, selector: &str, message: Option<Message>, mut callback: F) -> Result<()>
	where
		F: FnMut(QueryableMsg) -> Result<()> + Sized,
	{
		let replies = message
			.map_or_else(
				|| self.session.get(selector),
				|msg| self.session.get(selector).payload(msg.value()),
			)
			.consolidation(ConsolidationMode::None)
			.target(QueryTarget::All);

		#[cfg(feature = "unstable")]
		let replies = replies.allowed_destination(Locality::Any);

		let replies = replies
			//.timeout(Duration::from_millis(1000))
			.wait()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.result() {
				Ok(sample) => match sample.kind() {
					SampleKind::Put => {
						let content: Vec<u8> = sample.payload().into();
						callback(QueryableMsg(content))?;
					}
					SampleKind::Delete => {
						println!("Delete in Query");
					}
				},
				Err(err) => {
					let content: Vec<u8> = err.payload().into();
					println!(">> Received (ERROR: '{:?}' for {})", &content, &selector);
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
	async fn communicator_create() -> Result<()> {
		let cfg = dimas_config::Config::default();
		let _peer = Communicator::new(&cfg)?;
		Ok(())
	}
}
