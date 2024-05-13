// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

// region:		--- modules
use crate::{com::task_signal::TaskSignal, prelude::*};
use dimas_core::traits::{ManageState, OperationState};
use std::{fmt::Debug, sync::Mutex, time::Duration};
use tokio::{task::JoinHandle, time};
use tracing::{error, info, instrument, warn, Level};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> = Arc<
	Mutex<Option<Box<dyn FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static>>>,
>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`TimerBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`TimerBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`Timer`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, Timer<P>>>>,
}

/// State signaling that the [`TimerBuilder`] has no key expression set
pub struct NoKeyExpression;
#[allow(clippy::module_name_repetitions)]
/// State signaling that the [`TimerBuilder`] has the key expression set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`TimerBuilder`] has no interval set
pub struct NoInterval;
/// State signaling that the [`TimerBuilder`] has the interval set
pub struct Interval {
	/// The [`Duration`] of [`Timer`]s interval
	interval: Duration,
}

/// State signaling that the [`TimerBuilder`] has no interval callback set
pub struct NoIntervalCallback;
/// State signaling that the [`TimerBuilder`] has the interval callback set
pub struct IntervalCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The interval callback for the [`Timer`]
	pub callback: TimerCallback<P>,
}
// endregion:	--- states

// region:		--- TimerBuilder
/// A builder for a timer
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct TimerBuilder<P, K, I, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	context: ArcContext<P>,
	activation_state: OperationState,
	key_expr: K,
	interval: I,
	callback: C,
	storage: S,
	delay: Option<Duration>,
}

impl<P> TimerBuilder<P, NoKeyExpression, NoInterval, NoIntervalCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `TimerBuilder` in initial state
	#[must_use]
	pub const fn new(context: ArcContext<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Active,
			key_expr: NoKeyExpression,
			interval: NoInterval,
			callback: NoIntervalCallback,
			storage: NoStorage,
			delay: None,
		}
	}
}

impl<P, K, I, C, S> TimerBuilder<P, K, I, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the consolidation mode
	#[must_use]
	pub fn delay(mut self, delay: Duration) -> Self {
		self.delay.replace(delay);
		self
	}
}

impl<P, I, C, S> TimerBuilder<P, NoKeyExpression, I, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the key expression for the timer
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> TimerBuilder<P, KeyExpression, I, C, S> {
		let Self {
			context,
			activation_state,
			interval,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			interval,
			callback,
			storage,
			delay,
		}
	}

	/// Set only the name of the timer.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn name(self, topic: &str) -> TimerBuilder<P, KeyExpression, I, C, S> {
		let key_expr = self
			.context
			.prefix()
			.clone()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			context,
			activation_state,
			interval,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			key_expr: KeyExpression { key_expr },
			interval,
			callback,
			storage,
			delay,
		}
	}
}

impl<P, K, C, S> TimerBuilder<P, K, NoInterval, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// set timers interval
	#[must_use]
	pub fn interval(self, interval: Duration) -> TimerBuilder<P, K, Interval, C, S> {
		let Self {
			context,
			activation_state,
			key_expr: name,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			key_expr: name,
			interval: Interval { interval },
			callback,
			storage,
			delay,
		}
	}
}

impl<P, K, I, S> TimerBuilder<P, K, I, NoIntervalCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set interval callback for timer
	#[must_use]
	pub fn callback<F>(self, callback: F) -> TimerBuilder<P, K, I, IntervalCallback<P>, S>
	where
		F: FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			key_expr: name,
			interval,
			storage,
			delay,
			..
		} = self;
		let callback: TimerCallback<P> = Arc::new(Mutex::new(Some(Box::new(callback))));
		TimerBuilder {
			context,
			activation_state,
			key_expr: name,
			interval,
			callback: IntervalCallback { callback },
			storage,
			delay,
		}
	}
}

impl<P, K, I, C> TimerBuilder<P, K, I, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the timer
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Timer<P>>>>,
	) -> TimerBuilder<P, K, I, C, Storage<P>> {
		let Self {
			context,
			activation_state,
			key_expr: name,
			interval,
			callback,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			key_expr: name,
			interval,
			callback,
			storage: Storage { storage },
			delay,
		}
	}
}

impl<P, S> TimerBuilder<P, KeyExpression, Interval, IntervalCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [Timer]
	/// # Errors
	///
	pub fn build(self) -> Result<Timer<P>> {
		let Self {
			context,
			activation_state,
			key_expr: name,
			interval,
			callback,
			delay,
			..
		} = self;

		Ok(Timer::new(
			name.key_expr,
			context,
			activation_state,
			callback.callback,
			interval.interval,
			delay,
		))
	}
}

impl<P> TimerBuilder<P, KeyExpression, Interval, IntervalCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the timer to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<Option<Timer<P>>> {
		let name = self.key_expr.key_expr.clone();
		let collection = self.storage.storage.clone();
		let t = self.build()?;

		let r = collection
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(name, t);
		Ok(r)
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
		key_expr: String,
		/// Context for the Timer
		context: ArcContext<P>,
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
		key_expr: String,
		/// Context for the Timer
		context: ArcContext<P>,
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

impl<P> ManageState for Timer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_state(&mut self, state: &OperationState) -> Result<()> {
		match self {
			Self::Interval {
				key_expr: _,
				context: _,
				activation_state,
				interval: _,
				callback: _,
				handle,
			}
			| Self::DelayedInterval {
				key_expr: _,
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
		context: ArcContext<P>,
		activation_state: OperationState,
		callback: TimerCallback<P>,
		interval: Duration,
		delay: Option<Duration>,
	) -> Self {
		match delay {
			Some(delay) => Self::DelayedInterval {
				key_expr: name,
				context,
				activation_state,
				delay,
				interval,
				callback,
				handle: None,
			},
			None => Self::Interval {
				key_expr: name,
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
				key_expr,
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

				let key = key_expr.clone();
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle.replace(tokio::task::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("interval timer panic: {}", reason);
						if let Err(reason) = ctx1
							.tx
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
				key_expr,
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

				let key = key_expr.clone();
				let delay = *delay;
				let interval = *interval;
				let cb = callback.clone();
				let ctx1 = context.clone();
				let ctx2 = context.clone();

				handle.replace(tokio::task::spawn(async move {
					std::panic::set_hook(Box::new(move |reason| {
						error!("delayed timer panic: {}", reason);
						if let Err(reason) = ctx1
							.tx
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
				key_expr: _,
				context: _,
				activation_state: _,
				interval: _,
				callback: _,
				handle,
			}
			| Self::DelayedInterval {
				key_expr: _,
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
async fn run_timer<P>(interval: Duration, cb: TimerCallback<P>, ctx: ArcContext<P>)
where
	P: Send + Sync + Unpin + 'static,
{
	let mut interval = time::interval(interval);
	loop {
		interval.tick().await;

		match cb.lock() {
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
		is_normal::<TimerBuilder<Props, NoKeyExpression, NoInterval, NoIntervalCallback, NoStorage>>(
		);
	}
}
