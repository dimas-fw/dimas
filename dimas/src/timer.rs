// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

// region:		--- modules
use crate::{agent::TaskSignal, prelude::*};
use std::{fmt::Debug, sync::{mpsc::Sender, Mutex}, time::Duration};
use tokio::{task::JoinHandle, time};
use tracing::{error, instrument, warn, Level};
#[cfg(feature = "timer")]
use tracing::info;
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> = Arc<
	Mutex<Option<Box<dyn FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static>>>,
>;
// endregion:	--- types

// region:		--- TimerBuilder
/// A builder for a timer
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) context: ArcContext<P>,
	pub(crate) name: Option<String>,
	pub(crate) delay: Option<Duration>,
	pub(crate) interval: Option<Duration>,
	pub(crate) callback: Option<TimerCallback<P>>,
}

impl<P> TimerBuilder<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// set timers name
	#[must_use]
	pub fn name(mut self, name: &str) -> Self {
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
		F: FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		self.callback
			.replace(Arc::new(Mutex::new(Some(Box::new(callback)))));
		self
	}

	/// Build a timer
	/// # Errors
	///
	pub fn build(self) -> Result<Timer<P>> {
		let name = if self.name.is_none() {
			return Err(DimasError::NoName.into());
		} else {
			self.name.ok_or(DimasError::ShouldNotHappen)?
		};
		let interval = if self.interval.is_none() {
			return Err(DimasError::NoInterval.into());
		} else {
			self.interval.ok_or(DimasError::ShouldNotHappen)?
		};
		if self.callback.is_none() {
			return Err(DimasError::NoCallback.into());
		};

		match self.delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				name,
				delay,
				interval,
				callback: self.callback,
				handle: None,
				context: self.context,
			}),
			None => Ok(Timer::Interval {
				name,
				interval,
				callback: self.callback,
				handle: None,
				context: self.context,
			}),
		}
	}

	/// Build and add the timer to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "timer")))]
	#[cfg(feature = "timer")]
	pub fn add(self) -> Result<()> {
		let name = if self.name.is_none() {
			return Err(DimasError::NoName.into());
		} else {
			self.name.clone().ok_or(DimasError::ShouldNotHappen)?
		};
		let c = self.context.timers.clone();
		let timer = self.build()?;
		c.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(name, timer);
		Ok(())
	}
}
// endregion:	--- TimerBuilder

// region:		--- Timer
/// Timer
pub enum Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// A Timer with an Interval
	Interval {
		/// The Timers ID
		name: String,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// Timers Callback function called, when Timer is fired
		callback: Option<TimerCallback<P>>,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
		/// The agents Context available within the callback function
		context: ArcContext<P>,
	},
	/// A delayed Timer with an Interval
	DelayedInterval {
		/// The Timers ID
		name: String,
		/// The delay after which the first firing of the Timer happenes
		delay: Duration,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// Timers Callback function called, when Timer is fired
		callback: Option<TimerCallback<P>>,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
		/// The agents Context available within the callback function
		context: ArcContext<P>,
	},
}

impl<P> Debug for Timer<P>
where
	P: Send + Sync + Unpin + 'static,
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
	P: Send + Sync + Unpin + 'static,
{
	/// Start or restart the timer
	/// An already running timer will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn start(&mut self, tx: Sender<TaskSignal>) {
		self.stop();

		#[cfg(not(feature = "timer"))]
		drop(tx);

		match self {
			Self::Interval {
				name,
				interval,
				callback,
				handle,
				context,
			} => {
				{
					if let Some(cb) = callback.clone() {
						if let Err(err) = cb.lock() {
							warn!("found poisoned put Mutex");
							callback.replace(Arc::new(Mutex::new(err.into_inner().take())));
						}
					}
				}

				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();

				#[cfg(not(feature = "timer"))]
				let _key = name.clone();
				#[cfg(feature = "timer")]
				let key = name.clone();
				handle.replace(tokio::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("interval timer panic: {}", reason);
						#[cfg(feature = "timer")]
						if let Err(reason) = tx.send(TaskSignal::RestartTimer(key.clone())) {
							error!("could not restart timer: {}", reason);
						} else {
							info!("restarting timer!");
						};
					}));
					run_timer(interval, cb, ctx).await;
				}));
			}
			Self::DelayedInterval {
				name,
				delay,
				interval,
				callback,
				handle,
				context,
			} => {
				{
					if let Some(cb) = callback.clone() {
						if let Err(err) = cb.lock() {
							warn!("found poisoned put Mutex");
							callback.replace(Arc::new(Mutex::new(err.into_inner().take())));
						}
					}
				}

				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx = context.clone();

				#[cfg(not(feature = "timer"))]
				let _key = name.clone();
				#[cfg(feature = "timer")]
				let key = name.clone();
				handle.replace(tokio::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("delayed timer panic: {}", reason);
						#[cfg(feature = "timer")]
						if let Err(reason) = tx.send(TaskSignal::RestartTimer(key.clone())) {
							error!("could not restart timer: {}", reason);
						} else {
							info!("restarting timer!");
						};
					}));
					tokio::time::sleep(delay).await;
					run_timer(interval, cb, ctx).await;
				}));
			}
		}
	}

	/// Stop a running Timer
	#[instrument(level = Level::TRACE, skip_all)]
	pub fn stop(&mut self) {
		match self {
			Self::Interval {
				name: _,
				interval: _,
				callback: _,
				handle,
				context: _,
			}
			| Self::DelayedInterval {
				name: _,
				delay: _,
				interval: _,
				callback: _,
				handle,
				context: _,
			} => {
				if let Some(handle) = handle.take() {
					handle.abort();
				}
			}
		}
	}
}

#[instrument(name="timer", level = Level::ERROR, skip_all)]
async fn run_timer<P>(interval: Duration, cb: Option<TimerCallback<P>>, ctx: ArcContext<P>)
where
	P: Send + Sync + Unpin + 'static,
{
	let mut interval = time::interval(interval);
	loop {
		interval.tick().await;

		if let Some(cb) = cb.clone() {
			let result = cb.lock();
			match result {
				Ok(mut cb) => {
					if let Err(error) = cb.as_deref_mut().expect("snh")(&ctx) {
						error!("callback failed with {error}");
					}
				}
				Err(err) => {
					error!("callback lock failed with {err}");
				}
			}
		}
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
