//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};

use tokio::{task::JoinHandle, time::sleep};
use zenoh::config::{self, Config};

use crate::{
	com::{
		communicator::Communicator,
		publisher::PublisherBuilder,
		queryable::QueryableBuilder,
		subscriber::{SubscriberBuilder, SubscriberCallback},
	},
	timer::{TimerBuilder, TimerCollection},
};
// endregion: --- modules

// region:    --- Agent
/// Composable Agent
pub struct Agent<'a> {
	com: Arc<Communicator<'a>>,
	timers: TimerCollection,
	handles: Vec<JoinHandle<()>>,
}

impl<'a> Agent<'a> {
	pub fn new(config: Config, prefix: impl Into<String>) -> Self {
		let com = Arc::new(Communicator::new(config, prefix));
		Self {
			com,
			timers: Arc::new(RwLock::new(Vec::new())),
			handles: Vec::new(),
		}
	}

	pub fn uuid(&self) -> String {
		self.com.uuid()
	}

	pub async fn liveliness(&mut self) {
		self.com.liveliness().await;
	}

	pub async fn liveliness_subscriber(&self, callback: SubscriberCallback) {
		self.com.add_liveliness_subscriber(callback).await;
	}

	pub fn publisher(&self) -> PublisherBuilder<'a> {
		PublisherBuilder::default().communicator(self.com.clone())
	}

	pub fn subscriber(&self) -> SubscriberBuilder<'a> {
		SubscriberBuilder::default().communicator(self.com.clone())
	}

	pub fn queryable(&self) -> QueryableBuilder<'a> {
		QueryableBuilder::default().communicator(self.com.clone())
	}

	pub fn timer(&self) -> TimerBuilder {
		TimerBuilder::default()
			.collection(self.timers.clone())
			.session(self.com.session())
	}

	pub async fn start(&mut self) {
		self.com.start().await;
		for timer in self.timers.read().unwrap().iter() {
			let h = timer.write().unwrap().start();
			if let Some(h) = h {
				self.handles.push(h);
			}
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
