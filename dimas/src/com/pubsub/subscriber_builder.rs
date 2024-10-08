// Copyright © 2023 Stephan Kunz

//! Module `subscriber` provides a message `Subscriber` which can be created using the `SubscriberBuilder`.
//! A `Subscriber` can optional subscribe on a delete message.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use crate::com::pubsub::subscriber::Subscriber;
use crate::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage};
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Message,
	traits::Context,
	utils::selector_from,
};
use futures::future::{BoxFuture, Future};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
// endregion:	--- modules

// region:    	--- types
/// Type definition for a subscribers `put` callback
type PutCallback<P> =
	Box<dyn FnMut(Context<P>, Message) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `put` callback
pub type ArcPutCallback<P> = Arc<Mutex<PutCallback<P>>>;
/// Type definition for a subscribers `delete` callback
type DeleteCallback<P> = Box<dyn FnMut(Context<P>) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// Type definition for a subscribers atomic reference counted `delete` callback
pub type ArcDeleteCallback<P> = Arc<Mutex<DeleteCallback<P>>>;
// endregion: 	--- types

// region:		--- SubscriberBuilder
/// A builder for a subscriber
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	context: Context<P>,
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_origin: Locality,
	undeclare_on_drop: bool,
	selector: K,
	put_callback: C,
	storage: S,
	delete_callback: Option<ArcDeleteCallback<P>>,
}

impl<P> SubscriberBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct a `SubscriberBuilder` in initial state
	#[must_use]
	pub const fn new(context: Context<P>) -> Self {
		Self {
			context,
			activation_state: OperationState::Standby,
			#[cfg(feature = "unstable")]
			allowed_origin: Locality::Any,
			undeclare_on_drop: true,
			selector: NoSelector,
			put_callback: NoCallback,
			storage: NoStorage,
			delete_callback: None,
		}
	}
}

impl<P, K, C, S> SubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the allowed origin.
	#[cfg(feature = "unstable")]
	#[must_use]
	pub const fn allowed_origin(mut self, allowed_origin: Locality) -> Self {
		self.allowed_origin = allowed_origin;
		self
	}

	/// Set undeclare on drop.
	#[must_use]
	pub const fn undeclare_on_drop(mut self, undeclare_on_drop: bool) -> Self {
		self.undeclare_on_drop = undeclare_on_drop;
		self
	}

	/// Set subscribers callback for `delete` messages
	#[must_use]
	pub fn delete_callback<CB, F>(mut self, mut callback: CB) -> Self
	where
		CB: FnMut(Context<P>) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let callback: DeleteCallback<P> = Box::new(move |ctx| Box::pin(callback(ctx)));
		self.delete_callback
			.replace(Arc::new(Mutex::new(callback)));
		self
	}
}

impl<P, C, S> SubscriberBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full key expression for the [`Subscriber`].
	#[must_use]
	pub fn selector(self, selector: &str) -> SubscriberBuilder<P, Selector, C, S> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			storage,
			put_callback,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			selector: Selector {
				selector: selector.into(),
			},
			put_callback,
			storage,
			delete_callback,
		}
	}

	/// Set only the message qualifing part of the [`Subscriber`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> SubscriberBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> SubscriberBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set callback for put messages
	#[must_use]
	pub fn put_callback<CB, F>(
		self,
		mut callback: CB,
	) -> SubscriberBuilder<P, K, Callback<ArcPutCallback<P>>, S>
	where
		CB: FnMut(Context<P>, Message) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			selector,
			storage,
			delete_callback,
			..
		} = self;
		let callback: PutCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcPutCallback<P> = Arc::new(Mutex::new(callback));
		SubscriberBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			selector,
			put_callback: Callback { callback },
			storage,
			delete_callback,
		}
	}
}

impl<P, K, C> SubscriberBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the subscriber
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, Subscriber<P>>>>,
	) -> SubscriberBuilder<P, K, C, Storage<Subscriber<P>>> {
		let Self {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			selector,
			put_callback,
			delete_callback,
			..
		} = self;
		SubscriberBuilder {
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			selector,
			put_callback,
			storage: Storage { storage },
			delete_callback,
		}
	}
}

impl<P, S> SubscriberBuilder<P, Selector, Callback<ArcPutCallback<P>>, S>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Subscriber`].
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<Subscriber<P>> {
		let Self {
			selector,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			put_callback,
			delete_callback,
			..
		} = self;
		Ok(Subscriber::new(
			selector.selector,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_origin,
			undeclare_on_drop,
			put_callback.callback,
			delete_callback,
		))
	}
}

impl<P> SubscriberBuilder<P, Selector, Callback<ArcPutCallback<P>>, Storage<Subscriber<P>>>
where
	P: Send + Sync + 'static,
{
	/// Build and add the [`Subscriber`] to the [`Agent`].
	///
	/// # Errors
	/// Currently none
	pub fn add(self) -> Result<Option<Subscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.selector().to_string(), s);
		Ok(r)
	}
}
// endregion:	--- SubscriberBuilder

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<SubscriberBuilder<Props, NoSelector, NoCallback, NoStorage>>();
	}
}
