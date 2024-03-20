// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

// region:		--- modules
use crate::{prelude::*, utils::TaskSignal};
use std::{
	fmt::Debug,
	sync::{mpsc::Sender, Mutex},
	time::Duration,
};
use tokio::{task::JoinHandle, time};
#[cfg(feature = "timer")]
use tracing::info;
use tracing::{error, instrument, warn, Level};
// endregion:	--- modules

// region:		--- types
/// type definition for the functions called by a timer
#[allow(clippy::module_name_repetitions)]
pub type TimerCallback<P> = Arc<
	Mutex<Option<Box<dyn FnMut(&ArcContext<P>) -> Result<()> + Send + Sync + Unpin + 'static>>>,
>;
// endregion:	--- types

// region:		--- states
pub struct NoStorage;
#[cfg(feature = "timer")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub storage: Arc<RwLock<std::collections::HashMap<String, Timer<P>>>>,
}

pub struct NoKeyExpression;
#[allow(clippy::module_name_repetitions)]
pub struct KeyExpression {
	key_expr: String,
}

pub struct NoInterval;
pub struct Interval {
	interval: Duration,
}

pub struct NoIntervalCallback;
pub struct IntervalCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
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
	pub(crate) context: ArcContext<P>,
	pub(crate) key_expr: K,
	pub(crate) interval: I,
	pub(crate) callback: C,
	pub(crate) storage: S,
	pub(crate) delay: Option<Duration>,
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
			interval,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
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
		let key_expr = self.context.key_expr(topic);
		let Self {
			context,
			interval,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
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
			key_expr: name,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
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
			key_expr: name,
			interval,
			storage,
			delay,
			..
		} = self;
		let callback: TimerCallback<P> = Arc::new(Mutex::new(Some(Box::new(callback))));
		TimerBuilder {
			context,
			key_expr: name,
			interval,
			callback: IntervalCallback { callback },
			storage,
			delay,
		}
	}
}

#[cfg(feature = "timer")]
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
			key_expr: name,
			interval,
			callback,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
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
	/// Build the timer
	/// # Errors
	///
	pub fn build(self) -> Result<Timer<P>> {
		let Self {
			context,
			key_expr: name,
			interval,
			callback,
			delay,
			..
		} = self;

		match delay {
			Some(delay) => Ok(Timer::DelayedInterval {
				context,
				name: name.key_expr,
				delay,
				interval: interval.interval,
				callback: Some(callback.callback),
				handle: None,
			}),
			None => Ok(Timer::Interval {
				context,
				name: name.key_expr,
				interval: interval.interval,
				callback: Some(callback.callback),
				handle: None,
			}),
		}
	}
}

#[cfg(feature = "timer")]
impl<P> TimerBuilder<P, KeyExpression, Interval, IntervalCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the timer to the agents context
	/// # Errors
	///
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "timer")))]
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
				handle.replace(tokio::task::spawn(async move {
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
				handle.replace(tokio::task::spawn(async move {
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
		is_normal::<TimerBuilder<Props, NoKeyExpression, NoInterval, NoIntervalCallback, NoStorage>>(
		);
	}
}
