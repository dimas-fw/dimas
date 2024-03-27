// Copyright © 2024 Stephan Kunz

// region:		--- modules
use crate::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Mutex;
use tokio::task::JoinHandle;
// endregion:	--- modules

// region:		--- types
/// Type definition for a [`RosSubscribers`] callback function
#[allow(clippy::module_name_repetitions)]
pub type RosSubscriberCallback<P> = Arc<
	Mutex<Box<dyn FnMut(&ArcContext<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static>>,
>;
// endregion:	--- types

// region:		--- states
/// State signaling that the [`RosSubscriberBuilder`] has no storage value set
pub struct NoStorage;
/// State signaling that the [`RosSubscriberBuilder`] has the storage value set
#[cfg(feature = "ros_subscriber")]
pub struct Storage<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Thread safe reference to a [`HashMap`] to store the created [`RosSubscriber`]
	pub storage: Arc<RwLock<std::collections::HashMap<String, RosSubscriber<P>>>>,
}

/// State signaling that the [`RosSubscriberBuilder`] has no key expression value set
pub struct NoKeyExpression;
/// State signaling that the [`RosSubscriberBuilder`] has the key expression value set
pub struct KeyExpression {
	/// The key expression
	key_expr: String,
}

/// State signaling that the [`RosSubscriberBuilder`] has no callback value set
pub struct NoCallback;
/// State signaling that the [`RosSubscriberBuilder`] has the callback value set
pub struct Callback<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Callback for the [`RosSubscriber`]
	pub callback: RosSubscriberCallback<P>,
}
// endregion:	--- states

// region:		--- RosSubscriberBuilder
/// `RosSubscriberBuilder`
#[allow(clippy::module_name_repetitions)]
pub struct RosSubscriberBuilder<P, K, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	prefix: Option<String>,
	pub(crate) key_expr: K,
	pub(crate) callback: C,
	pub(crate) storage: S,
	phantom: PhantomData<P>,
}

impl<P> RosSubscriberBuilder<P, NoKeyExpression, NoCallback, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Construct a `RosSubscriberBuilder` in initial state
	#[must_use]
	pub const fn new(prefix: Option<String>) -> Self {
		Self {
			prefix,
			key_expr: NoKeyExpression,
			callback: NoCallback,
			storage: NoStorage,
			phantom: PhantomData,
		}
	}
}

impl<P, C, S> RosSubscriberBuilder<P, NoKeyExpression, C, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set the full key expression for the [`RosSubscriber`]
	#[must_use]
	pub fn key_expr(self, key_expr: &str) -> RosSubscriberBuilder<P, KeyExpression, C, S> {
		let Self {
			prefix,
			storage,
			callback,
			phantom,
			..
		} = self;
		RosSubscriberBuilder {
			prefix,
			key_expr: KeyExpression {
				key_expr: key_expr.into(),
			},
			callback,
			storage,
			phantom,
		}
	}

	/// Set only the message qualifing part of the [`RosSubscriber`].
	/// Will be prefixed with [`Agent`]s prefix.
	#[must_use]
	pub fn topic(mut self, topic: &str) -> RosSubscriberBuilder<P, KeyExpression, C, S> {
		let key_expr = self
			.prefix
			.take()
			.map_or(topic.to_string(), |prefix| format!("{prefix}/{topic}"));
		let Self {
			prefix,
			storage,
			callback,
			phantom,
			..
		} = self;
		RosSubscriberBuilder {
			prefix,
			key_expr: KeyExpression { key_expr },
			callback,
			storage,
			phantom,
		}
	}
}

impl<P, K, S> RosSubscriberBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Set callback for messages
	#[must_use]
	pub fn callback<F>(self, callback: F) -> RosSubscriberBuilder<P, K, Callback<P>, S>
	where
		F: FnMut(&ArcContext<P>, Message) -> Result<()> + Send + Sync + Unpin + 'static,
	{
		let Self {
			prefix,
			key_expr,
			storage,
			phantom,
			..
		} = self;
		let callback: RosSubscriberCallback<P> = Arc::new(Mutex::new(Box::new(callback)));
		RosSubscriberBuilder {
			prefix,
			key_expr,
			callback: Callback { callback },
			storage,
			phantom,
		}
	}
}

#[cfg(feature = "ros_subscriber")]
impl<P, K, C> RosSubscriberBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Provide [`Agent`]s storage for the [`RosSubscriber`]
	#[must_use]
	pub fn storage(
		self,
		storage: Arc<RwLock<std::collections::HashMap<String, RosSubscriber<P>>>>,
	) -> RosSubscriberBuilder<P, K, C, Storage<P>> {
		let Self {
			prefix,
			key_expr,
			callback,
			phantom,
			..
		} = self;
		RosSubscriberBuilder {
			prefix,
			key_expr,
			callback,
			storage: Storage { storage },
			phantom,
		}
	}
}

impl<P, S> RosSubscriberBuilder<P, KeyExpression, Callback<P>, S>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build the [`RosSubscriber`]
	///
	/// # Errors
	/// Currently none
	pub fn build(self) -> Result<RosSubscriber<P>> {
		let Self {
			key_expr, callback, ..
		} = self;
		Ok(RosSubscriber {
			key_expr: key_expr.key_expr,
			callback: callback.callback,
			handle: None,
		})
	}
}

#[cfg(feature = "ros_subscriber")]
impl<P> RosSubscriberBuilder<P, KeyExpression, Callback<P>, Storage<P>>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Build and add the [`RosSubscriber`] to the [`Agent`]
	///
	/// # Errors
	/// Currently none
	#[cfg_attr(any(nightly, docrs), doc, doc(cfg(feature = "ros_subscriber")))]
	pub fn add(self) -> Result<Option<RosSubscriber<P>>> {
		let c = self.storage.storage.clone();
		let s = self.build()?;

		let r = c
			.write()
			.map_err(|_| DimasError::ShouldNotHappen)?
			.insert(s.key_expr.clone(), s);
		Ok(r)
	}
}
// endregion:	--- RosSubscriberBuilder

// region:		--- RosSubscriber
/// `RosSubscriber`
pub struct RosSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	pub(crate) key_expr: String,
	callback: RosSubscriberCallback<P>,
	handle: Option<JoinHandle<()>>,
}

impl<P> Debug for RosSubscriber<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RosSubscriber")
			.field("key_expr", &self.key_expr)
			.finish_non_exhaustive()
	}
}
// endregion:	--- RosSubscriber

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<RosSubscriber<Props>>();
		is_normal::<RosSubscriberBuilder<Props, NoKeyExpression, NoCallback, NoStorage>>();
	}
}
