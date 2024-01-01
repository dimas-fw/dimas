//! Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::{
	com::{
		communicator::Communicator,
		publisher::{Publisher, PublisherBuilder},
		queryable::{Queryable, QueryableBuilder},
		subscriber::{Subscriber, SubscriberBuilder},
	},
	timer::{Timer, TimerBuilder},
};
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::time::sleep;
use zenoh::{config::Config, liveliness::LivelinessToken};
// endregion:	--- modules

// region:		--- Agent
pub struct Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	com: Arc<Communicator>,
	// an optional liveliness token
	liveliness_token: RwLock<Option<LivelinessToken<'a>>>,
	// an optional liveliness subscriber
	liveliness_subscriber: RwLock<Option<Arc<zenoh::subscriber::Subscriber<'a, ()>>>>,
	// registered subscribers
	subscribers: Arc<RwLock<Vec<Subscriber<P>>>>,
	// registered queryables
	queryables: Arc<RwLock<Vec<Queryable<P>>>>,
	// registered publisher
	publishers: Arc<RwLock<Vec<Publisher<'a>>>>,
	// registered timer
	timers: Arc<RwLock<Vec<Timer<P>>>>,
	// The agents propertie structure
	props: Arc<RwLock<P>>,
}

impl<'a, P> Agent<'a, P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn new(config: Config, prefix: impl Into<String>, properties: P) -> Self {
		let com = Arc::new(Communicator::new(config, prefix));
		Self {
			com,
			liveliness_token: RwLock::new(None),
			liveliness_subscriber: RwLock::new(None),
			subscribers: Arc::new(RwLock::new(Vec::new())),
			queryables: Arc::new(RwLock::new(Vec::new())),
			publishers: Arc::new(RwLock::new(Vec::new())),
			timers: Arc::new(RwLock::new(Vec::new())),
			props: Arc::new(RwLock::new(properties)),
		}
	}

	pub fn uuid(&self) -> String {
		self.com.uuid()
	}

	pub async fn liveliness(&mut self) {
		let token: LivelinessToken<'a> = self.com.liveliness().await;
		self.liveliness_token.write().unwrap().replace(token);
	}

	pub async fn liveliness_subscriber(&self, callback: fn(zenoh::sample::Sample)) {
		let subscriber = Arc::new(self.com.liveliness_subscriber(callback).await);
		self.liveliness_subscriber
			.write()
			.unwrap()
			.replace(subscriber);
	}

	pub fn subscriber(&self) -> SubscriberBuilder<P> {
		SubscriberBuilder {
			collection: Some(self.subscribers.clone()),
			communicator: Some(self.com.clone()),
			props: Some(self.props.clone()),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	pub fn queryable(&self) -> QueryableBuilder<P> {
		QueryableBuilder {
			collection: Some(self.queryables.clone()),
			communicator: Some(self.com.clone()),
			props: Some(self.props.clone()),
			key_expr: None,
			msg_type: None,
			callback: None,
		}
	}

	pub fn publisher(&self) -> PublisherBuilder<'a> {
		PublisherBuilder::default()
			.collection(self.publishers.clone())
			.communicator(self.com.clone())
	}

	pub fn timer(&self) -> TimerBuilder<P>
	where
		P: Default,
	{
		TimerBuilder::default()
			.collection(self.timers.clone())
			.communicator(self.com.clone())
			.properties(self.props.clone())
	}

	pub async fn start(&mut self) {
		for subscriber in self.subscribers.write().unwrap().iter_mut() {
			let _res = subscriber.start();
		}
		for timer in self.timers.write().unwrap().iter_mut() {
			let _res = timer.start();
		}
		loop {
			sleep(Duration::from_secs(1)).await;
		}
	}

	pub async fn stop(&mut self) {
		for timer in self.timers.write().unwrap().iter_mut() {
			let _res = timer.stop();
		}
		for subscriber in self.subscribers.write().unwrap().iter_mut() {
			let _res = subscriber.stop();
		}
	}
}
// endregion:	--- Agent

#[cfg(test)]
mod tests {
	use super::*;
	use zenoh::config;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	struct Props {}

	#[test]
	fn normal_types() {
		is_normal::<Agent<Props>>();
	}

	#[tokio::test]
	//#[serial]
	async fn agent_create_default() {
		let _agent1: Agent<Props> = Agent::new(config::peer(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(config::peer(), "agent2", Props {});
	}

	#[tokio::test(flavor = "current_thread")]
	//#[serial]
	async fn agent_create_current() {
		let _agent1: Agent<Props> = Agent::new(config::peer(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(config::peer(), "agent2", Props {});
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	//#[serial]
	async fn agent_create_restricted() {
		let _agent1: Agent<Props> = Agent::new(config::peer(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(config::peer(), "agent2", Props {});
	}

	#[tokio::test(flavor = "multi_thread")]
	//#[serial]
	async fn agent_create_multi() {
		let _agent1: Agent<Props> = Agent::new(config::peer(), "agent1", Props {});
		let _agent2: Agent<Props> = Agent::new(config::peer(), "agent2", Props {});
	}
}
