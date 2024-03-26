// Copyright © 2023 Stephan Kunz

//! [`Communicator`] implements the communication capabilities for an [`Agent`].
//!
//! # Examples
//! ```rust,no_run
//! # use dimas::prelude::*;
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() -> Result<()> {
//! # let agent = agent::new({});
//! # Ok(())
//! # }
//! ```
//!

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
use tracing::error;
use zenoh::prelude::{r#async::*, sync::SyncResolve};
use zenoh::publication::Publisher;
// endregion:	--- modules

// region:		--- Communicator
/// [`Communicator`] handles all communication aspects
#[derive(Debug)]
pub struct Communicator {
	/// The zenoh session
	pub(crate) session: Arc<Session>,
	/// A prefix to separate communication for different groups of [`Agent`]s
	pub(crate) prefix: Option<String>,
}

impl Communicator {
	/// Constructor
	pub(crate) fn new(config: crate::config::Config) -> Result<Self> {
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

	/// Get [`Agent`]s globally unique ID
	pub(crate) fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	/// Get [`Agent`]s group prefix
	pub(crate) fn prefix(&self) -> Option<String> {
		self.prefix.clone()
	}

	/// Set [`Agent`]s group prefix
	pub(crate) fn set_prefix(&mut self, prefix: impl Into<String>) {
		self.prefix = Some(prefix.into());
	}

	/// Create a key expression from a topic by adding [`Agent`]s prefix if one is given.
	#[must_use]
	pub fn key_expr(&self, topic: &str) -> String {
		self.prefix()
			.map_or_else(|| topic.into(), |prefix| format!("{prefix}/{topic}"))
	}

	/// Create a zenoh publisher
	pub(crate) fn create_publisher<'a>(&self, key_expr: &str) -> Result<Publisher<'a>> {
		let p = self
			.session
			.declare_publisher(key_expr.to_owned())
			.res_sync()
			.map_err(DimasError::DeclarePublisher)?;
		Ok(p)
	}

	/// Send an ad hoc put `message` of type `M` using the given `topic`.
	#[allow(clippy::needless_pass_by_value)]
	pub(crate) fn put<M>(&self, topic: &str, message: M) -> Result<()>
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
	pub(crate) fn delete(&self, topic: &str) -> Result<()> {
		let key_expr = self.key_expr(topic);

		self.session
			.delete(&key_expr)
			.res_sync()
			.map_err(|_| DimasError::Delete.into())
	}

	/// Send an ad hoc query using the given `query_name`.
	/// Response will be handled by `callback`, a closure or function with
	/// signature Fn(&[`ArcContext`]<AgentProperties>, [`Message`]).
	pub(crate) fn get<P, F>(
		&self,
		ctx: ArcContext<P>,
		query_name: &str,
		mode: ConsolidationMode,
		callback: F,
	) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
		F: Fn(&ArcContext<P>, Message) + Send + Sync + Unpin + 'static,
	{
		let key_expr = self.key_expr(query_name);
		let ctx = ctx;
		let session = self.session.clone();

		let replies = session
			.get(&key_expr)
			.consolidation(mode)
			//.timeout(Duration::from_millis(1000))
			.res_sync()
			.map_err(|_| DimasError::Get)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let msg = Message(sample);
						callback(&ctx, msg);
					}
					SampleKind::Delete => {
						println!("Delete in Query");
					}
				},
				Err(err) => error!(">> query receive error: {err})"),
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
		let mut peer1 = Communicator::new(crate::config::Config::default())?;
		peer1.set_prefix("peer1");
		Ok(())
	}
}
