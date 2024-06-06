// Copyright © 2023 Stephan Kunz

//! Module `timer` provides a set of `Timer` variants which can be created using the `TimerBuilder`.
//! When fired, a `Timer` calls his assigned `TimerCallback`.

// region:		--- modules
use super::timer::{Timer, TimerCallback};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::Context,
};
use std::{
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};
// endregion:	--- modules

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

/// State signaling that the [`TimerBuilder`] has no selector set
pub struct NoSelector;
#[allow(clippy::module_name_repetitions)]
/// State signaling that the [`TimerBuilder`] has the selector set
pub struct Selector {
	/// The selector
	selector: String,
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
	context: Context<P>,
	activation_state: OperationState,
	selector: K,
	interval: I,
	callback: C,
	storage: S,
	delay: Option<Duration>,
}

impl<P> TimerBuilder<P, NoSelector, NoInterval, NoIntervalCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `TimerBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Active,
			selector: NoSelector,
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

impl<P, I, C, S> TimerBuilder<P, NoSelector, I, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the key expression for the timer
	#[must_use]
	pub fn selector(self, selector: &str) -> TimerBuilder<P, Selector, I, C, S> {
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
			selector: Selector {
				selector: selector.into(),
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
	pub fn name(self, topic: &str) -> TimerBuilder<P, Selector, I, C, S> {
		let selector = self
			.context
			.prefix()
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
			selector: Selector { selector },
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
			selector: name,
			callback,
			storage,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			selector: name,
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
		F: FnMut(&Context<P>) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			context,
			activation_state,
			selector: name,
			interval,
			storage,
			delay,
			..
		} = self;
		let callback: TimerCallback<P> = Arc::new(Mutex::new(callback));
		TimerBuilder {
			context,
			activation_state,
			selector: name,
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
			selector: name,
			interval,
			callback,
			delay,
			..
		} = self;
		TimerBuilder {
			context,
			activation_state,
			selector: name,
			interval,
			callback,
			storage: Storage { storage },
			delay,
		}
	}
}

impl<P, S> TimerBuilder<P, Selector, Interval, IntervalCallback<P>, S>
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
			selector: name,
			interval,
			callback,
			delay,
			..
		} = self;

		Ok(Timer::new(
			name.selector,
			context,
			activation_state,
			callback.callback,
			interval.interval,
			delay,
		))
	}
}

impl<P> TimerBuilder<P, Selector, Interval, IntervalCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the timer to the agents context
	/// # Errors
	///
	pub fn add(self) -> Result<Option<Timer<P>>> {
		let name = self.selector.selector.clone();
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

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<TimerBuilder<Props, NoSelector, NoInterval, NoIntervalCallback, NoStorage>>();
	}
}
