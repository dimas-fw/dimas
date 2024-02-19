// Copyright © 2023 Stephan Kunz

//! Module Timer provides a set of timer which can be creating using the TimerBuilder.
//! When fired, the Timer calls his assigned TimerCallback

// region:		--- modules
use crate::{com::communicator::Communicator, context::Context, error::Result};
use std::{
	sync::{Arc, RwLock},
	time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle, time};
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

	/// Build a timer
	/// # Errors
	///
	/// # Panics
	///
	pub fn build(self) -> Result<Timer<P>> {
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
/// Timer
pub enum Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// A Timer with an Interval
	Interval {
		/// The interval in which the Timer is fired
		interval: Duration,
		/// Timers Callback function called, when Timer is fired
		callback: TimerCallback<P>,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
		/// The Context available within the callback function
		context: Arc<Context>,
		/// The Agents properties, available in the callback function
		props: Arc<RwLock<P>>,
	},
	/// A delayed Timer with an Interval
	DelayedInterval {
		/// The delay after which the first firing of the Timer happenes
		delay: Duration,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// Timers Callback function called, when Timer is fired
		callback: TimerCallback<P>,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
		/// The Context available within the callback function
		context: Arc<Context>,
		/// The Agents properties, available in the callback function
		props: Arc<RwLock<P>>,
	},
}

impl<P> Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Start Timer
	/// # Panics
	///
	pub fn start(&mut self) {
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
					let mut interval = time::interval(interval);
					loop {
						interval.tick().await;
						cb.lock().await(ctx.clone(), props.clone());
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
					let mut interval = time::interval(interval);
					loop {
						interval.tick().await;
						cb.lock().await(ctx.clone(), props.clone());
					}
				}));
			}
		}
	}

	/// Stop Timer
	/// # Panics
	///
	pub fn stop(&mut self) {
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
