// Copyright © 2023 Stephan Kunz

// region:		--- modules
use crate::{com::communicator::Communicator, context::Context, prelude::*};
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> =
	Arc<Mutex<dyn FnMut(Arc<Context>, Arc<RwLock<P>>) + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- TimerBuilder
/// A builder for a timer
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) collection: Arc<RwLock<Vec<Timer<P>>>>,
	pub(crate) communicator: Arc<Communicator>,
	pub(crate) delay: Option<Duration>,
	pub(crate) interval: Option<Duration>,
	pub(crate) callback: Option<TimerCallback<P>>,
	pub(crate) props: Arc<RwLock<P>>,
}

impl<P> TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// set timers delay
	#[must_use]
	pub fn delay(mut self, delay: Duration) -> Self {
		self.delay.replace(delay);
		self
	}

	/// set timers interval
	#[must_use]
	pub fn interval(mut self, interval: Duration) -> Self {
		self.interval.replace(interval);
		self
	}

	/// set timers callback function
	#[must_use]
	pub fn callback<F>(mut self, callback: F) -> Self
	where
		F: FnMut(Arc<Context>, Arc<RwLock<P>>) + Send + Sync + Unpin + 'static,
	{
		self.callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}

	pub(crate) fn build(self) -> Result<Timer<P>> {
		let interval = if self.interval.is_none() {
			return Err("No interval given".into());
		} else {
			self.interval.expect("should never happen")
		};
		let callback = if self.callback.is_none() {
			return Err("No callback given".into());
		} else {
			self.callback.expect("should never happen")
		};
		let props = self.props;
		let communicator = self.communicator;
		let ctx = Arc::new(Context { communicator });
		match self.delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				delay,
				interval,
				callback,
				handle: None,
				context: ctx,
				props,
			}),
			None => Ok(Timer::Interval {
				interval,
				callback,
				handle: None,
				context: ctx,
				props,
			}),
		}
	}

	/// add the timer to the agent
	/// # Errors
	///
	/// # Panics
	///
	pub fn add(self) -> Result<()> {
		let c = self.collection.clone();
		let timer = self.build()?;
		c.write()
			.expect("should never happen")
			.push(timer);
		Ok(())
	}
}
// endregion:	--- TimerBuilder

// region:		--- Timer
//#[derive(Debug, Clone)]
pub(crate) enum Timer<P>
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
	pub(crate) fn start(&mut self) {
		match self {
			Self::Interval {
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
			}
			Self::DelayedInterval {
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
			}
		}
	}

	pub(crate) fn stop(&mut self) {
		match self {
			Self::Interval {
				interval: _,
				callback: _,
				handle,
				context: _,
				props: _,
			}
			| Self::DelayedInterval {
				delay: _,
				interval: _,
				callback: _,
				handle,
				context: _,
				props: _,
			} => {
				handle
					.take()
					.expect("should never happen")
					.abort();
			}
		}
	}
}
// endregion:	--- Timer

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Default)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Timer<Props>>();
		is_normal::<TimerBuilder<Props>>();
	}
}
