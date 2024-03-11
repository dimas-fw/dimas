// Copyright © 2023 Stephan Kunz

//! Module `communicator` provides the `Communicator` implementing the communication capabilities for an `Agent`.

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
use tracing::error;
use zenoh::liveliness::LivelinessToken;
use zenoh::prelude::{r#async::*, sync::SyncResolve};
use zenoh::publication::Publisher;
// endregion:	--- modules

// region:		--- Communicator
#[derive(Debug)]
pub struct Communicator {
	/// the zenoh session
	pub(crate) session: Arc<Session>,
	/// prefix to separate agents communication
	pub(crate) prefix: Option<String>,
}

impl Communicator {
	pub(crate) fn new(config: crate::config::Config) -> Result<Self> {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.map_err(DimasError::SessionCreation)?,
		);
		Ok(Self {
			session,
			prefix: None,
		})
	}

	pub(crate) fn new_with_prefix(
		config: crate::config::Config,
		prefix: impl Into<String>,
	) -> Result<Self> {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.map_err(DimasError::SessionCreation)?,
		);
		let prefix = Some(prefix.into());
		Ok(Self { session, prefix })
	}

	pub(crate) fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	pub(crate) fn prefix(&self) -> Option<String> {
		self.prefix.clone()
	}

	pub(crate) fn key_expr(&self, msg_name: impl Into<String>) -> String {
		match self.prefix.clone() {
			Some(prefix) => prefix + "/" + &msg_name.into(),
			None => msg_name.into(),
		}
	}

	pub(crate) async fn send_liveliness<'a>(
		&self,
		msg_type: impl Into<String> + Send,
	) -> Result<LivelinessToken<'a>> {
		let session = self.session.clone();
		let uuid = self.key_expr(msg_type) + "/" + &session.zid().to_string();

		session
			.liveliness()
			.declare_token(&uuid)
			.res_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen.into())
	}

	pub(crate) fn create_publisher<'a>(
		&self,
		key_expr: impl Into<String> + Send,
	) -> Result<Publisher<'a>> {
		self.session
			.declare_publisher(key_expr.into())
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen.into())
	}

	#[allow(clippy::needless_pass_by_value)]
	pub(crate) fn put<M>(&self, msg_name: impl Into<String>, message: M) -> Result<()>
	where
		M: Encode,
	{
		let value: Vec<u8> = encode(&message);
		let key_expr = self.key_expr(msg_name);
		match self.session.put(&key_expr, value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::PutFailed.into()),
		}
	}

	pub(crate) fn delete(&self, msg_name: impl Into<String>) -> Result<()> {
		let key_expr = self.key_expr(msg_name);
		match self.session.delete(&key_expr).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::DeleteFailed.into()),
		}
	}

	pub(crate) fn get<P, F>(
		&self,
		ctx: ArcContext<P>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: F,
	) -> Result<()>
	where
		P: Debug + Send + Sync + Unpin + 'static,
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
			.map_err(|_| DimasError::ShouldNotHappen)?;

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

	#[tokio::test]
	//#[serial]
	async fn communicator_create_default() -> Result<()> {
		let _peer1 = Communicator::new(crate::config::Config::default());
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local()?, "peer2");
		//let _peer3 = Communicator::new(config::client());
		Ok(())
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn communicator_create_single() -> Result<()> {
		let _peer1 = Communicator::new(crate::config::Config::default());
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local()?, "peer2");
		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn communicator_create_restricted() -> Result<()> {
		let _peer1 = Communicator::new(crate::config::Config::default());
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local()?, "peer2");
		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn communicator_create_multi() -> Result<()> {
		let _peer1 = Communicator::new(crate::config::Config::default());
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local()?, "peer2");
		Ok(())
	}
}
