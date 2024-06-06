// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use super::liveliness::{ArcLivelinessCallback, LivelinessSubscriber};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::Context,
	utils::selector_from,
};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex, RwLock},
};
// endregion:	--- modules

// region:		--- states
/// State signaling that the [`LivelinessSubscriberBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`LivelinessSubscriberBuilder`] has the storage value set
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`LivelinessSubscriber`]
	pub storage: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
}

/// State signaling that the [`LivelinessSubscriberBuilder`] has no put callback set
pub struct NoPutCallback;
/// State signaling that the [`LivelinessSubscriberBuilder`] has the put callback set
pub struct PutCallback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The callback to use when receiving a put message
	pub callback: ArcLivelinessCallback<P>,
}
// endregion:	--- states

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: C,
	storage: S,
	delete_callback: Option<ArcLivelinessCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P, NoPutCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `LivelinessSubscriberBuilder` in initial state
	#[must_use]
	pub fn new(context: Context<P>) -> Self {
		//let token = context
		//	.prefix()
		//	.map_or("*".to_string(), |prefix| format!("{prefix}/*"));
		let token = selector_from("*", context.prefix());
		Self {
			token,
			context,
			activation_state: OperationState::Configured,
			put_callback: NoPutCallback,
			storage: NoStorage,
			delete_callback: None,
		}
	}
}

impl<P, C, S> LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set a different prefix for the liveliness subscriber.
	#[must_use]
	pub fn prefix(self, prefix: &str) -> Self {
		let token = format!("{prefix}/*");
		let Self {
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set an explicite token for the liveliness subscriber.
	#[must_use]
	pub fn token(self, token: impl Into<String>) -> Self {
		let Self {
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
			..
		} = self;
		Self {
			token: token.into(),
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set liveliness subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<F>(self, callback: F) -> Self
	where
		F: FnMut(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			..
		} = self;
		let delete_callback: Option<ArcLivelinessCallback<P>> =
			Some(Arc::new(Mutex::new(callback)));
		Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			delete_callback,
		}
	}
}

impl<P, S> LivelinessSubscriberBuilder<P, NoPutCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set liveliness subscribers callback for `put` messages
	#[must_use]
	pub fn put_callback<F>(self, callback: F) -> LivelinessSubscriberBuilder<P, PutCallback<P>, S>
	where
		F: FnMut(&Context<P>, &str) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			token,
			context,
			activation_state,
			storage,
			delete_callback,
			..
		} = self;
		let put_callback: ArcLivelinessCallback<P> = Arc::new(Mutex::new(callback));
		LivelinessSubscriberBuilder {
			token,
			context,
			activation_state,
			put_callback: PutCallback {
				callback: put_callback,
			},
			storage,
			delete_callback,
		}
	}
}

impl<P, C> LivelinessSubscriberBuilder<P, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide agents storage for the liveliness subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
	) -> LivelinessSubscriberBuilder<P, C, Storage<P>> {
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			..
		} = self;
		LivelinessSubscriberBuilder {
			token,
			context,
			activation_state,
			put_callback,
			storage: Storage { storage },
			delete_callback,
		}
	}
}

impl<P, S> LivelinessSubscriberBuilder<P, PutCallback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`LivelinessSubscriber`]
	/// # Errors
	///
	pub fn build(self) -> Result<LivelinessSubscriber<P>> {
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(LivelinessSubscriber::new(
			token,
			context,
			activation_state,
			put_callback.callback,
			delete_callback,
		))
	}
}

impl<P> LivelinessSubscriberBuilder<P, PutCallback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the liveliness subscriber to the agent
	/// # Errors
	///
	pub fn add(self) -> Result<Option<LivelinessSubscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.token(), s);
		Ok(r)
	}
}
// endregion:	--- LivelinessSubscriberBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<LivelinessSubscriberBuilder<Props, NoPutCallback, NoStorage>>();
	}
}
