//! Copyright © 2023 Stephan Kunz

use std::sync::Arc;

use zenoh::{
	liveliness::LivelinessToken,
	prelude::r#async::*,
	queryable::{Query, Queryable},
	subscriber::Subscriber,
};

#[derive(Debug)]
pub struct Communicator<'a> {
	// unique ID
	uuid: String,
	// the zenoh session
	session: Arc<Session>,
	// liveliness token
	_token: Option<LivelinessToken<'a>>,
	// registered subscribers
	subscriber: Vec<Subscriber<'a, ()>>,
	// registered queryables
	queryables: Vec<Queryable<'a, ()>>,
}

impl<'a> Communicator<'a> {
	pub fn new(config: Config, prefix: impl Into<String>) -> Self {
		tokio::task::block_in_place(move || {
			tokio::runtime::Handle::current().block_on(async move {
				let session = Arc::new(zenoh::open(config).res().await.unwrap());
				let session_clone = session.clone();
				let mut uuid = prefix.into();
				if !uuid.is_empty() {
					uuid += "/";
				}
				uuid += &session.zid().to_string();
				//dbg!(&uuid);
				let _token = Some(
					session_clone
						.liveliness()
						.declare_token(&uuid)
						.res()
						.await
						.unwrap(),
				);
				//dbg!(&_token);
				Self {
					uuid,
					session,
					_token,
					subscriber: Vec::new(),
					queryables: Vec::new(),
				}
			})
		})
	}

	pub fn uuid(&self) -> String {
		self.uuid.clone()
	}

	pub async fn add_subscriber(&mut self, key_expr: &str, fctn: fn(Sample)) {
		//dbg!(&key_expr);
		let s: zenoh::subscriber::Subscriber<'_, ()> = self
			.session
			.declare_subscriber(key_expr)
			.callback(fctn)
			.res()
			.await
			.unwrap();
		self.subscriber.push(s);
	}

	pub async fn add_queryable(&mut self, key_expr: &str, fctn: fn(Query)) {
		//dbg!(&key_expr);
		let q = self
			.session
			.declare_queryable(key_expr)
			.callback(fctn)
			.res()
			.await
			.unwrap();
		self.queryables.push(q);
	}
}

impl<'a> Default for Communicator<'a> {
	fn default() -> Self {
		Communicator::new(config::peer(), "peer")
	}
}

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
