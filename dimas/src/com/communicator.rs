//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use serde::Serialize;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use zenoh::prelude::{r#async::*, sync::SyncResolve};

#[cfg(feature = "query")]
use super::query::QueryCallback;
#[cfg(feature = "liveliness")]
use zenoh::liveliness::LivelinessToken;
#[cfg(feature = "publisher")]
use zenoh::publication::Publisher;
#[cfg(any(feature = "subscriber", feature = "liveliness"))]
use zenoh::subscriber::Subscriber;
// endregion:	--- modules

// region:		--- Communicator
#[derive(Debug)]
pub struct Communicator {
	// prefix to separate agents communicaton
	prefix: String,
	// the zenoh session
	session: Arc<Session>,
}

impl Communicator {
	pub fn new(config: crate::config::Config, prefix: impl Into<String>) -> Self {
		let cfg = config;
		let session = Arc::new(
			zenoh::open(cfg.zenoh_config())
				.res_sync()
				.expect("could not create zenoh session"),
		);
		let prefix = prefix.into();
		Self { prefix, session }
	}

	pub fn uuid(&self) -> String {
		self.session.zid().to_string()
	}

	pub fn prefix(&self) -> String {
		self.prefix.clone()
	}

	pub fn session(&self) -> Arc<Session> {
		self.session.clone()
	}
	#[cfg(feature = "liveliness")]
	pub async fn liveliness<'a>(&self, msg_type: impl Into<String>+Send) -> LivelinessToken<'a> {
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

	#[cfg(feature = "liveliness")]
	pub async fn liveliness_subscriber<'a>(&self, callback: fn(Sample)) -> Subscriber<'a, ()> {
		let key_expr = self.prefix.clone() + "/*";
		//dbg!(&key_expr);
		// create a liveliness subscriber
		let s = self
			.session
			.liveliness()
			.declare_subscriber(&key_expr)
			.callback(callback)
			.res_async()
			.await
			.expect("should never happen");

		// the initial liveliness query
		let replies = self
			.session
			.liveliness()
			.get(&key_expr)
			.timeout(Duration::from_millis(500))
			.res_async()
			.await
			.expect("should never happen");

		while let Ok(reply) = replies.recv_async().await {
			//dbg!(&reply);
			match reply.sample {
				Ok(sample) => {
					callback(sample);
				}
				Err(err) => println!(
					">> Received (ERROR: '{}')",
					String::try_from(&err).expect("to be implemented")
				),
			}
		}
		s
	}

	#[cfg(feature = "publisher")]
	pub async fn create_publisher<'a>(&self, key_expr: impl Into<String>+Send) -> Publisher<'a> {
		self.session
			.declare_publisher(key_expr.into())
			.res_async()
			.await
			.expect("should never happen")
	}

	#[cfg(feature = "publisher")]
	pub fn publish<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		let value = serde_json::to_string(&message).expect("should never happen");
		let key_expr =
			self.prefix.clone() + "/" + &msg_name.into() + "/" + &self.session.zid().to_string();
		//dbg!(&key_expr);
		match self.session().put(&key_expr, value).res_sync() {
			Ok(()) => Ok(()),
			Err(_) => Err("Context publish failed".into()),
		}
	}

	#[cfg(feature = "query")]
	pub fn query<P>(
		&self,
		ctx: Arc<Context>,
		props: Arc<RwLock<P>>,
		query_name: impl Into<String>,
		mode: ConsolidationMode,
		callback: QueryCallback<P>,
	)
	where
		P: Send + Sync + Unpin + 'static,
	{
		let key_expr = self.prefix.clone() + "/" + &query_name.into();
		//dbg!(&key_expr);
		let ctx = ctx;
		let props = props;
		let session = self.session();

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
					callback(ctx.clone(), props.clone(), sample);
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
