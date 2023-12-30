//! Copyright © 2023 Stephan Kunz

// region:    --- modules
use crate::{com::communicator::Communicator, context::Context, prelude::*};
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};
// endregion: --- modules

// region:    --- types
type TimerFctn = Arc<Mutex<dyn FnMut(Arc<Context>) + Send + Sync + Unpin + 'static>>;
// endregion: --- types

// region:    --- TimerBuilder
#[derive(Default, Clone)]
pub struct TimerBuilder {
	collection: Option<Arc<RwLock<Vec<Timer>>>>,
	communicator: Option<Arc<Communicator>>,
	delay: Option<Duration>,
	interval: Option<Duration>,
	callback: Option<TimerFctn>,
}

impl TimerBuilder {
	pub fn collection(mut self, collection: Arc<RwLock<Vec<Timer>>>) -> Self {
		self.collection.replace(collection);
		self
	}

	pub fn communicator(mut self, communicator: Arc<Communicator>) -> Self {
		self.communicator.replace(communicator);
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
		F: FnMut(Arc<Context>) + Send + Sync + Unpin + 'static,
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
		let mut ctx = Arc::new(Context::default());
		if self.communicator.is_some() {
			let communicator = self.communicator.unwrap();
			ctx = Arc::new(Context { communicator });
		}
		match self.delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				delay,
				interval: self.interval.unwrap(),
				fctn: self.callback.unwrap(),
				handle: None,
				ctx,
			}),
			None => Ok(Timer::Interval {
				interval: self.interval.unwrap(),
				fctn: self.callback.unwrap(),
				handle: None,
				ctx,
			}),
		}
	}

	pub fn add(mut self) -> Result<()> {
		if self.collection.is_none() {
			return Err("No collection given".into());
		}
		let c = self.collection.take();
		let timer = self.build()?;
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
		handle: Option<JoinHandle<()>>,
		ctx: Arc<Context>,
	},
	DelayedInterval {
		delay: Duration,
		interval: Duration,
		fctn: TimerFctn,
		handle: Option<JoinHandle<()>>,
		ctx: Arc<Context>,
	},
}

impl Timer {
	pub fn start(&mut self) -> Result<()> {
		match self {
			Timer::Interval {
				interval,
				fctn,
				handle,
				ctx,
			} => {
				let interval = *interval;
				let fctn = fctn.clone();
				let ctx = ctx.clone();
				handle.replace(tokio::spawn(async move {
					loop {
						let mut fctn = fctn.lock().await;
						fctn(ctx.clone());
						tokio::time::sleep(interval).await;
					}
				}));
				Ok(())
			}
			Timer::DelayedInterval {
				delay,
				interval,
				fctn,
				handle,
				ctx,
			} => {
				let delay = *delay;
				let interval = *interval;
				let fctn = fctn.clone();
				let ctx = ctx.clone();
				handle.replace(tokio::spawn(async move {
					tokio::time::sleep(delay).await;
					loop {
						let mut fctn = fctn.lock().await;
						fctn(ctx.clone());
						tokio::time::sleep(interval).await;
					}
				}));
				Ok(())
			}
		}
	}

	pub fn stop(&mut self) -> Result<()> {
		match self {
			Timer::Interval {
				interval: _,
				fctn: _,
				handle,
				ctx: _,
			} => {
				handle.take().unwrap().abort();
				Ok(())
			}
			Timer::DelayedInterval {
				delay: _,
				interval: _,
				fctn: _,
				handle,
				ctx: _,
			} => {
				handle.take().unwrap().abort();
				Ok(())
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
		timer.start().unwrap();
		tokio::time::sleep(Duration::from_millis(100)).await;
		timer.stop().unwrap();
		//let res = tokio::join!(h);
		//match res {
		//	(Ok(_),) => (),
		//	(Err(_),) => panic!("join error"),
		//}
	}
}
