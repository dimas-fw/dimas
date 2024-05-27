// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

// region:		--- modules
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::Result,
	traits::{Capability, Context},
};
use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
	time::Duration,
};
use tokio::{task::JoinHandle, time};
use tracing::{error, info, instrument, warn, Level};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> =
	Arc<Mutex<dyn FnMut(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static>>;
// endregion:	--- types

// region:		--- Timer
/// Timer
pub enum Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// A Timer with an Interval
	Interval {
		/// The Timers ID
		selector: String,
		/// Context for the Timer
		context: Context<P>,
		/// [`OperationState`] on which this timer is started
		activation_state: OperationState,
		/// Timers Callback function called, when Timer is fired
		callback: TimerCallback<P>,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
	},
	/// A delayed Timer with an Interval
	DelayedInterval {
		/// The Timers ID
		selector: String,
		/// Context for the Timer
		context: Context<P>,
		/// [`OperationState`] on which this timer is started
		activation_state: OperationState,
		/// Timers Callback function called, when Timer is fired
		callback: TimerCallback<P>,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// The delay after which the first firing of the Timer happenes
		delay: Duration,
		/// The handle to stop the Timer
		handle: Option<JoinHandle<()>>,
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

impl<P> Capability for Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		match self {
			Self::Interval {
				selector: _,
				context: _,
				activation_state,
				interval: _,
				callback: _,
				handle,
			}
			| Self::DelayedInterval {
				selector: _,
				context: _,
				activation_state,
				delay: _,
				interval: _,
				callback: _,
				handle,
			} => {
				if (state >= &activation_state) && handle.is_none() {
					return self.start();
				} else if (state < &activation_state) && handle.is_some() {
					self.stop();
					return Ok(());
				}
			}
		}
		Ok(())
	}
}

impl<P> Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [Timer]
	#[must_use]
	pub fn new(
		name: String,
		context: Context<P>,
		activation_state: OperationState,
		callback: TimerCallback<P>,
		interval: Duration,
		delay: Option<Duration>,
	) -> Self {
		match delay {
			Some(delay) => Self::DelayedInterval {
				selector: name,
				context,
				activation_state,
				delay,
				interval,
				callback,
				handle: None,
			},
			None => Self::Interval {
				selector: name,
				context,
				activation_state,
				interval,
				callback,
				handle: None,
			},
		}
	}

	/// Start or restart the timer
	/// An already running timer will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		match self {
			Self::Interval {
				selector,
				context,
				activation_state: _,
				interval,
				callback,
				handle,
			} => {
				{
					if callback.lock().is_err() {
						warn!("found poisoned Mutex");
						callback.clear_poison();
					}
				}

				let key = selector.clone();
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle.replace(tokio::task::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("interval timer panic: {}", reason);
						if let Err(reason) = ctx1
							.sender()
							.send(TaskSignal::RestartTimer(key.clone()))
						{
							error!("could not restart timer: {}", reason);
						} else {
							info!("restarting timer!");
						};
					}));
					run_timer(interval, cb, ctx2).await;
				}));
				Ok(())
			}
			Self::DelayedInterval {
				selector,
				context,
				activation_state: _,
				delay,
				interval,
				callback,
				handle,
			} => {
				{
					if callback.lock().is_err() {
						warn!("found poisoned Mutex");
						callback.clear_poison();
					}
				}

				let key = selector.clone();
				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle.replace(tokio::task::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("delayed timer panic: {}", reason);
						if let Err(reason) = ctx1
							.sender()
							.send(TaskSignal::RestartTimer(key.clone()))
						{
							error!("could not restart timer: {}", reason);
						} else {
							info!("restarting timer!");
						};
					}));
					tokio::time::sleep(delay).await;
					run_timer(interval, cb, ctx2).await;
				}));
				Ok(())
			}
		}
	}

	/// Stop a running Timer
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&mut self) {
		match self {
			Self::Interval {
				selector: _,
				context: _,
				activation_state: _,
				interval: _,
				callback: _,
				handle,
			}
			| Self::DelayedInterval {
				selector: _,
				context: _,
				activation_state: _,
				delay: _,
				interval: _,
				callback: _,
				handle,
			} => {
				if let Some(handle) = handle.take() {
					handle.abort();
				}
			}
		}
	}
}

#[instrument(name="timer", level = Level::ERROR, skip_all)]
async fn run_timer<P>(interval: Duration, cb: TimerCallback<P>, ctx: Context<P>)
where
	P: Send + Sync + Unpin + 'static,
{
	let mut interval = time::interval(interval);
	loop {
		interval.tick().await;

		match cb.lock() {
			Ok(mut cb) => {
				if let Err(error) = cb(&ctx) {
					error!("callback failed with {error}");
				}
			}
			Err(err) => {
				error!("callback lock failed with {err}");
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
	}
}
