// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use alloc::{boxed::Box, string::String, sync::Arc};
use anyhow::Result;
use core::{fmt::Debug, time::Duration};
use dimas_core::{enums::TaskSignal, traits::Context, OperationState, Operational};
#[cfg(feature = "std")]
use parking_lot::Mutex;
#[cfg(feature = "std")]
use tokio::{task::JoinHandle, time};
use tracing::{error, info, instrument, warn, Level};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
pub type ArcTimerCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>) -> Result<()> + Send + Sync + 'static>>;
// endregion:	--- types

// region:		--- Timer
/// Timer
pub enum Timer<P>
where
	P: Send + Sync + 'static,
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
		callback: ArcTimerCallback<P>,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// The handle to stop the Timer
		handle: Mutex<Option<JoinHandle<()>>>,
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
		callback: ArcTimerCallback<P>,
		/// The interval in which the Timer is fired
		interval: Duration,
		/// The delay after which the first firing of the Timer happenes
		delay: Duration,
		/// The handle to stop the Timer
		handle: Mutex<Option<JoinHandle<()>>>,
	},
}

impl<P> Debug for Timer<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl<P> Operational for Timer<P>
where
	P: Send + Sync + 'static,
{
	fn manage_operation_state_old(&self, state: OperationState) -> Result<()> {
		match *self {
			Self::Interval {
				selector: _,
				context: _,
				activation_state,
				interval: _,
				callback: _,
				handle: _,
			}
			| Self::DelayedInterval {
				selector: _,
				context: _,
				activation_state,
				delay: _,
				interval: _,
				callback: _,
				handle: _,
			} => {
				if state >= activation_state {
					self.start()
				} else if state < activation_state {
					self.stop()
				} else {
					Ok(())
				}
			}
		}
	}

	fn state(&self) -> OperationState {
		todo!()
	}

	fn set_state(&mut self, _state: OperationState) {
		todo!()
	}

	fn operationals(&mut self) -> &mut std::vec::Vec<Box<dyn Operational>> {
		todo!()
	}
}

impl<P> Timer<P>
where
	P: Send + Sync + 'static,
{
	/// Constructor for a [Timer]
	#[must_use]
	pub fn new(
		name: impl Into<String>,
		context: Context<P>,
		activation_state: OperationState,
		callback: ArcTimerCallback<P>,
		interval: Duration,
		delay: Option<Duration>,
	) -> Self {
		match delay {
			Some(delay) => Self::DelayedInterval {
				selector: name.into(),
				context,
				activation_state,
				delay,
				interval,
				callback,
				handle: Mutex::new(None),
			},
			None => Self::Interval {
				selector: name.into(),
				context,
				activation_state,
				interval,
				callback,
				handle: Mutex::new(None),
			},
		}
	}

	/// Start or restart the timer
	/// An already running timer will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&self) -> Result<()> {
		self.stop()?;

		match self {
			Self::Interval {
				selector,
				context,
				activation_state: _,
				interval,
				callback,
				handle,
			} => {
				let key = selector.clone();
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle
					.lock()
					.replace(tokio::task::spawn(async move {
						std::panic::set_hook(Box::new(move |reason| {
							error!("delayed timer panic: {}", reason);
							if let Err(reason) = ctx1
								.sender()
								.blocking_send(TaskSignal::RestartTimer(key.clone()))
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
				let key = selector.clone();
				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle
					.lock()
					.replace(tokio::task::spawn(async move {
						std::panic::set_hook(Box::new(move |reason| {
							error!("delayed timer panic: {}", reason);
							if let Err(reason) = ctx1
								.sender()
								.blocking_send(TaskSignal::RestartTimer(key.clone()))
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
	fn stop(&self) -> Result<()> {
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
				let handle = handle.lock().take();
				if let Some(handle) = handle {
					handle.abort();
				};
				Ok(())
			}
		}
	}
}

#[instrument(name="timer", level = Level::ERROR, skip_all)]
async fn run_timer<P>(interval: Duration, cb: ArcTimerCallback<P>, ctx: Context<P>)
where
	P: Send + Sync + 'static,
{
	let mut interval = time::interval(interval);
	loop {
		let ctx = ctx.clone();
		interval.tick().await;

		if let Err(error) = cb.lock()(ctx) {
			error!("callback failed with {error}");
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Timer<Props>>();
	}
}
