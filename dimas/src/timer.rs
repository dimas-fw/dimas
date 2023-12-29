//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use crate::prelude::*;
use serde::Serialize;
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};
use zenoh::prelude::sync::SyncResolve;
// endregion: --- modules

// region:    --- types
type TimerFctn = Arc<Mutex<dyn FnMut(Arc<TimerContext>) + Send + Sync + Unpin + 'static>>;
pub type TimerCollection = Arc<RwLock<Vec<Arc<RwLock<Timer>>>>>;
// endregion: --- types

// region:    --- TimerContext
#[derive(Debug, Default, Clone)]
pub struct TimerContext {
	session: Option<Arc<zenoh::Session>>,
}

impl TimerContext {
	pub fn publish<T>(&self, msg_name: impl Into<String>, message: T) -> Result<()>
	where
		T: Serialize,
	{
		let value = serde_json::to_string(&message).unwrap();
		let session = self.session.clone().unwrap();
		let key_expr =
			"nemo".to_string() + "/" + &msg_name.into() + "/" + &session.zid().to_string();
		//dbg!(&key_expr);
		match session.put(&key_expr, value).res_sync() {
			Ok(_) => Ok(()),
			Err(_) => Err("Timer publish failed".into()),
		}
	}
}
// endregion: --- TimerContext

// region:    --- TimerBuilder
#[derive(Default, Clone)]
pub struct TimerBuilder {
	collection: Option<TimerCollection>,
	session: Option<Arc<zenoh::Session>>,
	delay: Option<Duration>,
	interval: Option<Duration>,
	callback: Option<TimerFctn>,
}

impl TimerBuilder {
	pub fn collection(mut self, collection: TimerCollection) -> Self {
		self.collection.replace(collection);
		self
	}

	pub fn session(mut self, session: Arc<zenoh::Session>) -> Self {
		self.session.replace(session);
		self
	}

	pub fn delay(mut self, delay: Duration) -> Self {
		self.delay.replace(delay);
		self
	}

	pub fn interval(mut self, interval: Duration) -> Self {
		self.interval.replace(interval);
		self
	}

	pub fn callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(Arc<TimerContext>) + Send + Sync + Unpin + 'static,
	{
		self.callback.replace(Arc::new(Mutex::new(callback)));
		self
	}

	pub(crate) fn build(self) -> Result<Timer> {
		if self.interval.is_none() {
			return Err("No interval given".into());
		}
		if self.callback.is_none() {
			return Err("No callback given".into());
		}
		let mut ctx = Arc::new(TimerContext::default());
		if self.session.is_some() {
			let session = Some(self.session.unwrap());
			ctx = Arc::new(TimerContext { session });
		}
		match self.delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				delay,
				interval: self.interval.unwrap(),
				fctn: self.callback.unwrap(),
				ctx,
			}),
			None => Ok(Timer::Interval {
				interval: self.interval.unwrap(),
				fctn: self.callback.unwrap(),
				ctx,
			}),
		}
	}

	pub fn add(mut self) -> Result<()> {
		if self.collection.is_none() {
			return Err("No collection given".into());
		}
		let c = self.collection.take();
		let timer = Arc::new(RwLock::new(self.build()?));
		c.unwrap().write().unwrap().push(timer);
		Ok(())
	}
}
// endregion: --- TimerBuilder

// region:    --- Timer
//#[derive(Debug, Clone)]
pub enum Timer {
	Interval {
		interval: Duration,
		fctn: TimerFctn,
		ctx: Arc<TimerContext>,
	},
	DelayedInterval {
		delay: Duration,
		interval: Duration,
		fctn: TimerFctn,
		ctx: Arc<TimerContext>,
	},
}

impl Timer {
	//pub fn context(&self) -> Arc<TimerContext> {
	//}

	pub fn start(&mut self) -> Option<JoinHandle<()>> {
		match self {
			Timer::Interval {
				interval,
				fctn,
				ctx,
			} => {
				let interval = *interval;
				let fctn = fctn.clone();
				let ctx = ctx.clone();
				Some(tokio::spawn(async move {
					loop {
						let mut fctn = fctn.lock().await;
						fctn(ctx.clone());
						tokio::time::sleep(interval).await;
					}
				}))
			}
			Timer::DelayedInterval {
				delay,
				interval,
				fctn,
				ctx,
			} => {
				let delay = *delay;
				let interval = *interval;
				let fctn = fctn.clone();
				let ctx = ctx.clone();
				Some(tokio::spawn(async move {
					tokio::time::sleep(delay).await;
					loop {
						let mut fctn = fctn.lock().await;
						fctn(ctx.clone());
						tokio::time::sleep(interval).await;
					}
				}))
			}
		}
	}
}
// endregion: --- Timer

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Timer>();
		is_normal::<TimerBuilder>();
		is_normal::<TimerContext>();
	}

	#[test]
	fn timer_create() {
		let _timer1 = TimerBuilder::default()
			.interval(Duration::from_millis(100))
			.callback(|_data| {
				dbg!();
			})
			.build()
			.unwrap();
		//assert!(timer1.context().session());

		let _timer2 = TimerBuilder::default()
			.delay(Duration::from_millis(10000))
			.interval(Duration::from_millis(100))
			.callback(|data| {
				dbg!(&data);
			})
			.build()
			.unwrap();
	}

	#[tokio::test]
	async fn timer_start() {
		let mut timer = TimerBuilder::default()
			.interval(Duration::from_millis(100))
			.callback(|data| {
				dbg!(&data);
			})
			.build()
			.unwrap();
		let h = timer.start().unwrap();
		tokio::time::sleep(Duration::from_millis(100)).await;
		h.abort();
		//let res = tokio::join!(h);
		//match res {
		//	(Ok(_),) => (),
		//	(Err(_),) => panic!("join error"),
		//}
	}
}
