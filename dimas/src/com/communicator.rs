// Copyright © 2023 Stephan Kunz

// region:		--- modules
use super::query::QueryCallback;
use crate::prelude::*;
use std::fmt::Debug;
use zenoh::liveliness::LivelinessToken;
use zenoh::prelude::{r#async::*, sync::SyncResolve};
use zenoh::publication::Publisher;
// endregion:	--- modules

// region:		--- Communicator
#[derive(Debug)]
pub struct Communicator {
	/// the zenoh session
	pub(crate) session: Arc<Session>,
	/// prefix to separate agents communicaton
	pub(crate) prefix: Option<String>,
}

impl Communicator {
	pub(crate) fn new(config: crate::config::Config) -> Self {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.expect("could not create zenoh session"),
		);
		Self {
			session,
			prefix: None,
		}
	}

	pub(crate) fn new_with_prefix(
		config: crate::config::Config,
		prefix: impl Into<String>,
	) -> Self {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.expect("could not create zenoh session"),
		);
		let prefix = Some(prefix.into());
		Self { session, prefix }
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
	) -> LivelinessToken<'a> {
		let session = self.session.clone();
		let uuid = self.key_expr(msg_type) + "/" + &session.zid().to_string();
		//dbg!(&uuid);
		session
			.liveliness()
			.declare_token(&uuid)
			.res_async()
			.await
			.expect("should never happen")
	}

	pub(crate) fn create_publisher<'a>(&self, key_expr: impl Into<String> + Send) -> Publisher<'a> {
		self.session
			.declare_publisher(key_expr.into())
			.res_sync()
			.expect("should never happen")
	}

	pub(crate) fn put<M>(&self, msg_name: impl Into<String>, message: M) -> Result<()>
	where
		M: bitcode::Encode,
	{
		let value: Vec<u8> = bitcode::encode(&message).expect("should never happen");
		let key_expr = self.key_expr(msg_name);
		//dbg!(&key_expr);
		match self.session.put(&key_expr, value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Publish failed".into()),
		}
	}

	pub(crate) fn delete(&self, msg_name: impl Into<String>) -> Result<()> {
		let key_expr = self.key_expr(msg_name);
		//dbg!(&key_expr);
		match self.session.delete(&key_expr).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Delete failed".into()),
		}
	}

	pub(crate) fn get<P>(
		&self,
		ctx: Arc<Context<P>>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: QueryCallback<P>,
	) where
		P: Debug + Send + Sync + Unpin + 'static,
	{
		let key_expr = self.key_expr(query_name);
		//dbg!(&key_expr);
		let ctx = ctx;
		let props = props;
		let session = self.session.clone();

		let replies = session
			.get(&key_expr)
			.consolidation(mode)
			//.timeout(Duration::from_millis(1000))
			.res_sync()
			.expect("should never happen");
		//dbg!(&replies);

		while let Ok(reply) = replies.recv() {
			//dbg!(&reply);
			match reply.sample {
				Ok(sample) => {
					//dbg!(&sample);
					let value: Vec<u8> = sample
						.value
						.try_into()
						.expect("should not happen");

					let msg = Message {
						key_expr: sample.key_expr.to_string(),
						value,
					};

					match sample.kind {
						SampleKind::Put => {
							callback(&ctx, &props, &msg);
						}
						SampleKind::Delete => {
							println!("Delete in Query");
						}
					}
				}
				Err(err) => println!(
					">> No data (ERROR: '{}')",
					String::try_from(&err).expect("to be implemented")
				),
			}
		}
	}
}

impl Default for Communicator {
	fn default() -> Self {
		Self::new(crate::config::Config::local())
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
	async fn communicator_create_default() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn communicator_create_single() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local(), "peer2");
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn communicator_create_restricted() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local(), "peer2");
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn communicator_create_multi() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new_with_prefix(crate::config::Config::local(), "peer2");
	}
}
