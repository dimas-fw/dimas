// Copyright © 2023 Stephan Kunz

// region:		--- modules
use std::sync::Arc;
use zenoh::prelude::{r#async::*, sync::SyncResolve};

#[cfg(feature = "query")]
use super::query::QueryCallback;
#[cfg(feature = "query")]
use crate::context::Context;
#[cfg(feature = "publisher")]
use crate::prelude::*;
#[cfg(feature = "query")]
use std::sync::RwLock;
#[cfg(feature = "liveliness")]
use zenoh::liveliness::LivelinessToken;
#[cfg(feature = "publisher")]
use zenoh::publication::Publisher;
// endregion:	--- modules

// region:		--- Communicator
#[derive(Debug)]
pub struct Communicator {
	// prefix to separate agents communicaton
	pub(crate) prefix: String,
	// the zenoh session
	pub(crate) session: Arc<Session>,
}

impl Communicator {
	pub(crate) fn new(config: crate::config::Config, prefix: impl Into<String>) -> Self {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.expect("could not create zenoh session"),
		);
		let prefix = prefix.into();
		Self { prefix, session }
	}

	pub(crate) fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	pub(crate) fn prefix(&self) -> String {
		self.prefix.clone()
	}

	//#[cfg_attr(doc, doc(cfg(feature = "liveliness")))]
	#[cfg(feature = "liveliness")]
	pub(crate) async fn liveliness<'a>(
		&self,
		msg_type: impl Into<String> + Send,
	) -> LivelinessToken<'a> {
		let session = self.session.clone();
		let uuid = self.prefix.clone() + "/" + &msg_type.into() + "/" + &session.zid().to_string();
		//dbg!(&uuid);
		session
			.liveliness()
			.declare_token(&uuid)
			.res_async()
			.await
			.expect("should never happen")
	}

	//#[cfg_attr(doc, doc(cfg(feature = "publisher")))]
	#[cfg(feature = "publisher")]
	pub(crate) async fn create_publisher<'a>(
		&self,
		key_expr: impl Into<String> + Send,
	) -> Publisher<'a> {
		self.session
			.declare_publisher(key_expr.into())
			.res_async()
			.await
			.expect("should never happen")
	}

	//#[cfg_attr(doc, doc(cfg(feature = "publisher")))]
	#[cfg(feature = "publisher")]
	pub(crate) fn publish<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: bincode::Encode,
	{
		let value = bincode::encode_to_vec(message, bincode::config::standard())
			.expect("should never happen");
		let key_expr = self.prefix.clone() + "/" + &msg_name.into();
		//dbg!(&key_expr);
		match self.session.put(&key_expr, value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Publish failed".into()),
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "publisher")))]
	#[cfg(feature = "publisher")]
	pub(crate) fn delete(&self, msg_name: impl Into<String>) -> Result<()> {
		let key_expr = self.prefix.clone() + "/" + &msg_name.into();
		//dbg!(&key_expr);
		match self.session.delete(&key_expr).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Delete failed".into()),
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "query")))]
	#[cfg(feature = "query")]
	pub fn query<P>(
		&self,
		ctx: Arc<Context>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: QueryCallback<P>,
	) where
		P: Send + Sync + Unpin + 'static,
	{
		let key_expr = self.prefix.clone() + "/" + &query_name.into();
		//dbg!(&key_expr);
		let ctx = ctx;
		let props = props;
		let session = self.session.clone();

		let replies = session
			.get(&key_expr)
			// ensure to get more than one interface from a host
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
					match sample.kind {
						SampleKind::Put => {
							callback(&ctx, &props, &value);
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
		Self::new(crate::config::Config::local(), "peer")
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
		let _peer2 = Communicator::new(crate::config::Config::local(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn communicator_create_single() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(crate::config::Config::local(), "peer2");
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn communicator_create_restricted() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(crate::config::Config::local(), "peer2");
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn communicator_create_multi() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(crate::config::Config::local(), "peer2");
	}
}
