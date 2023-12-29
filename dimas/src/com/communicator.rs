//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use super::{queryable::QueryableCallback, subscriber::SubscriberCallback};
use std::sync::{Arc, RwLock};
use zenoh::{
	liveliness::LivelinessToken, prelude::r#async::*, queryable::Queryable, subscriber::Subscriber,
};
// endregion: --- modules

// region:    --- Communicator
#[derive(Debug)]
pub struct Communicator<'a> {
	// prefix to separate agents communicaton
	prefix: String,
	// the zenoh session
	session: Arc<Session>,
	// an optional liveliness subscriber
	liveliness_subscriber: RwLock<Option<Arc<Subscriber<'a, ()>>>>,
	// liveliness token
	token: RwLock<Option<LivelinessToken<'a>>>,
	// registered subscribers
	subscribers: RwLock<Vec<Subscriber<'a, ()>>>,
	// registered queryables
	queryables: RwLock<Vec<Queryable<'a, ()>>>,
}

impl<'a> Communicator<'a> {
	pub fn new(config: Config, prefix: impl Into<String>) -> Self {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let session = Arc::new(zenoh::open(config).res().await.unwrap());
				let prefix = prefix.into();
				//dbg!(&_token);
				Self {
					prefix,
					session,
					liveliness_subscriber: RwLock::new(None),
					token: RwLock::new(None),
					subscribers: RwLock::new(Vec::new()),
					queryables: RwLock::new(Vec::new()),
				}
			})
		})
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

	pub async fn liveliness(&self) {
		let session = self.session.clone();
		let uuid = self.prefix.clone() + "/" + &session.zid().to_string();
		//dbg!(&uuid);
		let token = session
			.liveliness()
			.declare_token(&uuid)
			.res()
			.await
			.unwrap();
		self.token.write().unwrap().replace(token);
	}

	pub async fn add_liveliness_subscriber(&self, callback: SubscriberCallback) {
		let key_expr = String::from("nemo/*");
		//dbg!(&key_expr);
		// add a liveliness subscriber
		let s = Arc::new(
			self.session
				.liveliness()
				.declare_subscriber(&key_expr)
				.callback(callback)
				.res()
				.await
				.unwrap(),
		);
		self.liveliness_subscriber.write().unwrap().replace(s);

		// the initial liveliness query
		let replies = self
			.session
			.liveliness()
			.get(&key_expr)
			//.timeout(Duration::from_millis(500))
			.res()
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
	}

	pub fn add_subscriber(&self, key_expr: &str, callback: SubscriberCallback) {
		//dbg!(&key_expr);
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let s: zenoh::subscriber::Subscriber<'_, ()> = self
					.session
					.declare_subscriber(key_expr)
					.callback(callback)
					.res()
					.await
					.unwrap();
				self.subscribers.write().unwrap().push(s);
			})
		})
	}

	pub fn add_queryable(&self, key_expr: &str, callback: QueryableCallback) {
		//dbg!(&key_expr);
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let q = self
					.session
					.declare_queryable(key_expr)
					.callback(callback)
					.res()
					.await
					.unwrap();
				self.queryables.write().unwrap().push(q);
			})
		})
	}

	pub async fn start(&self) {}
}

impl<'a> Default for Communicator<'a> {
	fn default() -> Self {
		Communicator::new(config::peer(), "peer")
	}
}
// endregion: --- Communicator

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

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn communicator_create() {
		let _peer1 = Communicator::default();
		let _peer2 = Communicator::new(config::peer(), "peer2");
		//let _peer3 = Communicator::new(config::client());
	}
}
