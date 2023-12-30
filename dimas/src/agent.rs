//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use crate::{
	com::{
		communicator::Communicator,
		publisher::PublisherBuilder,
		queryable::QueryableBuilder,
		subscriber::{SubscriberBuilder, SubscriberCallback},
	},
	timer::{Timer, TimerBuilder},
};
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::time::sleep;
use zenoh::{
	config::{self, Config},
	liveliness::LivelinessToken,
	publication::Publisher,
	queryable::Queryable,
	subscriber::Subscriber,
};
// endregion: --- modules

// region:    --- AgentInner
struct AgentInner<'a> {
	// an optional liveliness token
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
	// an optional liveliness subscriber
	liveliness_subscriber: RwLock<Option<Arc<Subscriber<'a, ()>>>>,
	// registered subscribers
	subscribers: Arc<RwLock<Vec<Subscriber<'a, ()>>>>,
	// registered queryables
	queryables: Arc<RwLock<Vec<Queryable<'a, ()>>>>,
	// registered publisher
	publishers: Arc<RwLock<Vec<Publisher<'a>>>>,
	// registered timer
	timers: Arc<RwLock<Vec<Timer>>>,
}
// endregion: --- AgentInner

// region:    --- Agent
/// Composable Agent
pub struct Agent<'a> {
	com: Arc<Communicator>,
	inner: AgentInner<'a>,
}

impl<'a> Agent<'a> {
	pub fn new(config: Config, prefix: impl Into<String>) -> Self {
		let com = Arc::new(Communicator::new(config, prefix));
		Self {
			com,
			inner: AgentInner {
				liveliness_token: RwLock::new(None),
				liveliness_subscriber: RwLock::new(None),
				subscribers: Arc::new(RwLock::new(Vec::new())),
				queryables: Arc::new(RwLock::new(Vec::new())),
				publishers: Arc::new(RwLock::new(Vec::new())),
				timers: Arc::new(RwLock::new(Vec::new())),
			},
		}
	}

	pub fn uuid(&self) -> String {
		self.com.uuid()
	}

	pub async fn liveliness(&mut self) {
		let token: LivelinessToken<'a> = self.com.liveliness();
		self.inner.liveliness_token.write().unwrap().replace(token);
	}

	pub async fn liveliness_subscriber(&self, callback: SubscriberCallback) {
		let subscriber = Arc::new(self.com.liveliness_subscriber(callback).await);
		self.inner
			.liveliness_subscriber
			.write()
			.unwrap()
			.replace(subscriber);
	}

	pub fn subscriber(&self) -> SubscriberBuilder<'a> {
		SubscriberBuilder::default()
			.collection(self.inner.subscribers.clone())
			.communicator(self.com.clone())
	}

	pub fn queryable(&self) -> QueryableBuilder<'a> {
		QueryableBuilder::default()
			.collection(self.inner.queryables.clone())
			.communicator(self.com.clone())
	}

	pub fn publisher(&self) -> PublisherBuilder<'a> {
		PublisherBuilder::default()
			.collection(self.inner.publishers.clone())
			.communicator(self.com.clone())
	}

	pub fn timer(&self) -> TimerBuilder {
		TimerBuilder::default()
			.collection(self.inner.timers.clone())
			.communicator(self.com.clone())
	}

	pub async fn start(&mut self) {
		for timer in self.inner.timers.write().unwrap().iter_mut() {
			let _res = timer.start();
		}
		loop {
			sleep(Duration::from_secs(1)).await;
		}
	}
}

impl<'a> Default for Agent<'a> {
	fn default() -> Self {
		Agent::new(config::peer(), "agent")
	}
}
// endregion: --- Agent

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Agent>();
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn agent_create() {
		let _agent1 = Agent::default();
		let _agent2 = Agent::new(config::peer(), "agent2");
		//let _agent3 = Agent::new(config::client());
	}
}
