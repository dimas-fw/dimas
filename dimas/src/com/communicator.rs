//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use zenoh::{
	liveliness::LivelinessToken,
	prelude::{r#async::*, sync::SyncResolve},
	publication::Publisher,
	subscriber::Subscriber,
};
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
	pub fn new(config: Config, prefix: impl Into<String>) -> Self {
		let session = Arc::new(zenoh::open(config).res_sync().unwrap());
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

	pub async fn liveliness<'a>(&self) -> LivelinessToken<'a> {
		let session = self.session.clone();
		let uuid = self.prefix.clone() + "/" + &session.zid().to_string();
		//dbg!(&uuid);
		session
			.liveliness()
			.declare_token(&uuid)
			.res_async()
			.await
			.unwrap()
	}

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
			.unwrap();

		// the initial liveliness query
		let replies = self
			.session
			.liveliness()
			.get(&key_expr)
			.timeout(Duration::from_millis(500))
			.res_async()
			.await
			.unwrap();

		while let Ok(reply) = replies.recv_async().await {
			//dbg!(&reply);
			match reply.sample {
				Ok(sample) => {
					callback(sample);
				}
				Err(err) => println!(
					">> Received (ERROR: '{}')",
					String::try_from(&err).unwrap_or("".to_string())
				),
			}
		}
		s
	}

	pub async fn create_publisher<'a>(&self, key_expr: impl Into<String>) -> Publisher<'a> {
		self.session
			.declare_publisher(key_expr.into())
			.res_async()
			.await
			.unwrap()
	}

	pub fn publish<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		let value = serde_json::to_string(&message).unwrap();
		let key_expr =
			self.prefix.clone() + "/" + &msg_name.into() + "/" + &self.session.zid().to_string();
		//dbg!(&key_expr);
		match self.session().put(&key_expr, value).res_sync() {
			Ok(_) => Ok(()),
			Err(_) => Err("Context publish failed".into()),
		}
	}
}

impl Default for Communicator {
	fn default() -> Self {
		Communicator::new(config::peer(), "peer")
	}
}
// endregion:	--- Communicator

#[cfg(test)]
mod tests {
	use super::*;
	//use serial_test::serial;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Communicator>();
	}

	#[tokio::test]
	//#[serial]
	async fn communicator_create_default() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(config::peer(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn communicator_create_single() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(config::peer(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn communicator_create_restricted() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(config::peer(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn communicator_create_multi() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(config::peer(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}
}
