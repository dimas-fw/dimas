// Copyright © 2023 Stephan Kunz

//! Module Timer provides a set of timer which can be creating using the TimerBuilder.
//! When fired, the Timer calls his assigned TimerCallback

// region:		--- modules
use crate::prelude::*;
use std::{fmt::Debug, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> = Arc<Mutex<dyn FnMut(ArcContext<P>) + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- TimerBuilder
/// A builder for a timer
#[allow(clippy::module_name_repetitions)]
#[derive(Default, Clone)]
pub struct TimerBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) name: Option<String>,
	pub(crate) delay: Option<Duration>,
	pub(crate) interval: Option<Duration>,
	pub(crate) callback: Option<TimerCallback<P>>,
}

impl<P> TimerBuilder<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// set timers name
	#[must_use]
	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name.replace(name.into());
		self
	}

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
		F: FnMut(ArcContext<P>) + Send + Sync + Unpin + 'static,
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
			return Err(Error::NoInterval);
		} else {
			self.interval.expect("should never happen")
		};
		let callback = if self.callback.is_none() {
			return Err(Error::NoCallback);
		} else {
			self.callback.expect("should never happen")
		};

		match self.delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				delay,
				interval,
				callback,
				handle: None,
				context: self.context,
			}),
			None => Ok(Timer::Interval {
				interval,
				callback,
				handle: None,
				context: self.context,
			}),
		}
	}

	//#[cfg_attr(doc, doc(cfg(feature = "timer")))]
	/// add the timer to the agents context
	/// # Errors
	///
	/// # Panics
	///
	#[cfg(feature = "timer")]
	pub fn add(self) -> Result<()> {
		let name = if self.name.is_none() {
			return Err(Error::NoName);
		} else {
			self.name.clone().expect("should never happen")
		};
		let c = self.context.timers.clone();
		let timer = self.build()?;
		c.write()
			.expect("should never happen")
			.insert(name, timer);
		Ok(())
	}
}
// endregion:	--- TimerBuilder

// region:		--- Timer
/// Timer
pub enum Timer<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	/// A Timer with an Interval
	Interval {
		/// The interval in which the Timer is fired
		interval: Duration,
		/// Timers Callback function called, when Timer is fired
		callback: TimerCallback<P>,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
		/// The agents Context available within the callback function
		context: ArcContext<P>,
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
		/// The agents Context available within the callback function
		context: ArcContext<P>,
	},
}

impl<P> Debug for Timer<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Interval { interval, .. } => f
				.debug_struct("IntervalTimer")
				.field("interval", interval)
				.finish_non_exhaustive(),
			Self::DelayedInterval {
				delay, interval, ..
			} => f
				.debug_struct("DelayedIntervalTimer")
				.field("delay", delay)
				.field("interval", interval)
				.finish_non_exhaustive(),
		}
	}
}

impl<P> Timer<P>
where
	P: Debug + Send + Sync + Unpin + 'static,
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
			} => {
				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();
				handle.replace(tokio::spawn(async move {
					run_timer(interval, cb, ctx).await;
				}));
			}
			Self::DelayedInterval {
				delay,
				interval,
				callback,
				handle,
				context,
			} => {
				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();
				handle.replace(tokio::spawn(async move {
					tokio::time::sleep(delay).await;
					run_timer(interval, cb, ctx).await;
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
			}
			| Self::DelayedInterval {
				delay: _,
				interval: _,
				callback: _,
				handle,
				context: _,
			} => {
				handle
					.take()
					.expect("should never happen")
					.abort();
			}
		}
	}
}

use tracing::{span, Instrument, Level};

//#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn run_timer<P>(interval: Duration, cb: TimerCallback<P>, ctx: ArcContext<P>)
where
	P: Debug + Send + Sync + Unpin + 'static,
{
	let mut interval = time::interval(interval);
	loop {
		interval.tick().await;
		cb.lock()
			.instrument(span!(Level::INFO, "run_timer"))
			.await(ctx.clone());
	}
}
// endregion:	--- Timer

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Timer<Props>>();
		is_normal::<TimerBuilder<Props>>();
	}
}
