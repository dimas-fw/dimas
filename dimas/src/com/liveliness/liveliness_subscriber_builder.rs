// Copyright © 2023 Stephan Kunz

//! Module `liveliness` provides a `LivelinessSubscriber` which can be created using the `LivelinessSubscriberBuilder`.
//! A `LivelinessSubscriber` can optional subscribe on a delete message.

// region:		--- modules
use super::{
	liveliness_subscriber::LivelinessSubscriber, ArcLivelinessCallback, LivelinessCallback,
};
use crate::{Callback, NoCallback, NoStorage, Storage};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::Context,
	utils::selector_from,
};
use futures::future::Future;
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};
use tokio::sync::Mutex;
// endregion:	--- modules

// region:    	--- types
// endregion: 	--- types

// region:		--- LivelinessSubscriberBuilder
/// The builder for the liveliness subscriber
#[allow(clippy::module_name_repetitions)]
pub struct LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + 'static,
{
	token: String,
	context: Context<P>,
	activation_state: OperationState,
	put_callback: C,
	storage: S,
	delete_callback: Option<ArcLivelinessCallback<P>>,
}

impl<P> LivelinessSubscriberBuilder<P, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
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
			activation_state: OperationState::Created,
			put_callback: NoCallback,
			storage: NoStorage,
			delete_callback: None,
		}
	}
}

impl<P, C, S> LivelinessSubscriberBuilder<P, C, S>
where
	P: Send + Sync + 'static,
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
	pub fn delete_callback<CB, F>(self, mut callback: CB) -> Self
	where
		CB: FnMut(Context<P>, String) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			token,
			context,
			activation_state,
			put_callback,
			storage,
			..
		} = self;

		let callback: LivelinessCallback<P> =
			Box::new(move |ctx, txt| Box::pin(callback(ctx, txt)));
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

impl<P, S> LivelinessSubscriberBuilder<P, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set liveliness subscribers callback for `put` messages
	#[must_use]
	pub fn put_callback<CB, F>(
		self,
		mut callback: CB,
	) -> LivelinessSubscriberBuilder<P, Callback<ArcLivelinessCallback<P>>, S>
	where
		CB: FnMut(Context<P>, String) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			token,
			context,
			activation_state,
			storage,
			delete_callback,
			..
		} = self;
		let callback: LivelinessCallback<P> =
			Box::new(move |ctx, txt| Box::pin(callback(ctx, txt)));
		let put_callback: ArcLivelinessCallback<P> = Arc::new(Mutex::new(callback));
		LivelinessSubscriberBuilder {
			token,
			context,
			activation_state,
			put_callback: Callback {
				callback: put_callback,
			},
			storage,
			delete_callback,
		}
	}
}

impl<P, C> LivelinessSubscriberBuilder<P, C, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the liveliness subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<HashMap<String, LivelinessSubscriber<P>>>>,
	) -> LivelinessSubscriberBuilder<P, C, Storage<LivelinessSubscriber<P>>> {
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

impl<P, S> LivelinessSubscriberBuilder<P, Callback<ArcLivelinessCallback<P>>, S>
where
	P: Send + Sync + 'static,
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

impl<P>
	LivelinessSubscriberBuilder<
		P,
		Callback<ArcLivelinessCallback<P>>,
		Storage<LivelinessSubscriber<P>>,
	>
where
	P: Send + Sync + 'static,
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<LivelinessSubscriberBuilder<Props, NoCallback, NoStorage>>();
	}
}
