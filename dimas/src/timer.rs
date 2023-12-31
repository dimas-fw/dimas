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
type TimerCallback<P> =
	Arc<Mutex<dyn FnMut(Arc<Context>, Arc<RwLock<P>>) + Send + Sync + Unpin + 'static>>;
// endregion: --- types

// region:    --- TimerBuilder
#[derive(Default, Clone)]
pub struct TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	collection: Option<Arc<RwLock<Vec<Timer<P>>>>>,
	communicator: Option<Arc<Communicator>>,
	delay: Option<Duration>,
	interval: Option<Duration>,
	callback: Option<TimerCallback<P>>,
	props: Option<Arc<RwLock<P>>>,
}

impl<P> TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub fn collection(mut self, collection: Arc<RwLock<Vec<Timer<P>>>>) -> Self {
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
		F: FnMut(Arc<Context>, Arc<RwLock<P>>) + Send + Sync + Unpin + 'static,
	{
		self.callback.replace(Arc::new(Mutex::new(callback)));
		self
	}

	pub fn properties(mut self, properties: Arc<RwLock<P>>) -> Self {
		self.props.replace(properties);
		self
	}

	pub(crate) async fn build(self) -> Result<Timer<P>> {
		if self.interval.is_none() {
			return Err("No interval given".into());
		}
		if self.callback.is_none() {
			return Err("No callback given".into());
		}
		if self.props.is_none() {
			return Err("No properties given".into());
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
				callback: self.callback.unwrap(),
				handle: None,
				context: ctx,
				props: self.props.unwrap(),
			}),
			None => Ok(Timer::Interval {
				interval: self.interval.unwrap(),
				callback: self.callback.unwrap(),
				handle: None,
				context: ctx,
				props: self.props.unwrap(),
			}),
		}
	}

	pub async fn add(mut self) -> Result<()> {
		if self.collection.is_none() {
			return Err("No collection given".into());
		}
		let c = self.collection.take();
		let timer = self.build().await?;
		c.unwrap().write().unwrap().push(timer);
		Ok(())
	}
}
// endregion: --- TimerBuilder

// region:    --- Timer
//#[derive(Debug, Clone)]
pub enum Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	Interval {
		interval: Duration,
		callback: TimerCallback<P>,
		handle: Option<JoinHandle<()>>,
		context: Arc<Context>,
		props: Arc<RwLock<P>>,
	},
	DelayedInterval {
		delay: Duration,
		interval: Duration,
		callback: TimerCallback<P>,
		handle: Option<JoinHandle<()>>,
		context: Arc<Context>,
		props: Arc<RwLock<P>>,
	},
}

impl<T> Timer<T>
where
	T: Send + Sync + Unpin + 'static,
{
	pub fn start(&mut self) -> Result<()> {
		match self {
			Timer::Interval {
				interval,
				callback,
				handle,
				context,
				props,
			} => {
				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();
				let props = props.clone();
				handle.replace(tokio::spawn(async move {
					loop {
						cb.lock().await(ctx.clone(), props.clone());
						tokio::time::sleep(interval).await;
					}
				}));
				Ok(())
			}
			Timer::DelayedInterval {
				delay,
				interval,
				callback,
				handle,
				context,
				props,
			} => {
				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();
				let props = props.clone();
				handle.replace(tokio::spawn(async move {
					tokio::time::sleep(delay).await;
					loop {
						cb.lock().await(ctx.clone(), props.clone());
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
				callback: _,
				handle,
				context: _,
				props: _,
			} => {
				handle.take().unwrap().abort();
				Ok(())
			}
			Timer::DelayedInterval {
				delay: _,
				interval: _,
				callback: _,
				handle,
				context: _,
				props: _,
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

	#[derive(Default)]
	struct Props {}

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Timer<Props>>();
		is_normal::<TimerBuilder<Props>>();
	}

	#[tokio::test]
	async fn timer_create() {
		let test = Arc::new(RwLock::new(Props {}));
		let _timer1 = TimerBuilder::default()
			.interval(Duration::from_millis(100))
			.callback(|_data, _props| {
				dbg!();
			})
			.properties(test)
			.build()
			.await
			.unwrap();
		//assert!(timer1.context().session());

		let _timer2 = TimerBuilder::default()
			.delay(Duration::from_millis(10000))
			.interval(Duration::from_millis(100))
			.callback(|_data, _props| {
				dbg!();
			})
			.properties(Arc::new(RwLock::new(Props {})))
			.build()
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn timer_run() {
		let mut timer = TimerBuilder::default()
			.interval(Duration::from_millis(10))
			.callback(|_data, _props| {
				dbg!();
			})
			.properties(Arc::new(RwLock::new(Props {})))
			.build()
			.await
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
