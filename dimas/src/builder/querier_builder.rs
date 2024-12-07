// Copyright © 2023 Stephan Kunz

//! Module `query` provides an information/compute requestor `Query` which can be created using the `QuerierBuilder`.

// region:		--- modules
use anyhow::Result;
use dimas_com::zenoh::querier::{ArcGetCallback, GetCallback, Querier, QuerierParameter};
use dimas_core::{
	message_types::QueryableMsg, traits::Context, utils::selector_from, ActivityType, Component,
	ComponentType, OperationState, OperationalType,
};
use futures::Future;
use std::sync::Arc;
use tokio::{sync::Mutex, time::Duration};
#[cfg(feature = "unstable")]
use zenoh::sample::Locality;
use zenoh::{
	bytes::Encoding,
	query::{ConsolidationMode, QueryTarget},
};

use super::{
	builder_states::{Callback, NoCallback, NoSelector, NoStorage, Selector, Storage},
	error::Error,
};
// endregion:	--- modules

// region:		--- QuerierBuilder
/// The builder for a query
#[derive(Clone)]
pub struct QuerierBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	session_id: String,
	context: Context<P>,
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_destination: Locality,
	encoding: String,
	timeout: Duration,
	selector: K,
	callback: C,
	storage: S,
	mode: ConsolidationMode,
	target: QueryTarget,
}

impl<P> QuerierBuilder<P, NoSelector, NoCallback, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Construct a `QuerierBuilder` in initial state
	#[must_use]
	pub fn new(session_id: impl Into<String>, context: Context<P>) -> Self {
		Self {
			session_id: session_id.into(),
			context,
			activation_state: OperationState::Active,
			#[cfg(feature = "unstable")]
			allowed_destination: Locality::Any,
			encoding: Encoding::default().to_string(),
			timeout: Duration::from_millis(100),
			selector: NoSelector,
			callback: NoCallback,
			storage: NoStorage,
			mode: ConsolidationMode::None,
			target: QueryTarget::All,
		}
	}
}

impl<P, K, C, S> QuerierBuilder<P, K, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the activation state.
	#[must_use]
	pub const fn activation_state(mut self, state: OperationState) -> Self {
		self.activation_state = state;
		self
	}

	/// Set the [`ConsolidationMode`] of the [`Querier`].
	#[must_use]
	pub const fn mode(mut self, mode: ConsolidationMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set the session id.
	#[must_use]
	pub fn session_id(mut self, session_id: &str) -> Self {
		self.session_id = session_id.into();
		self
	}

	/// Set the [`QueryTarget`] of the [`Querier`].
	#[must_use]
	pub const fn target(mut self, target: QueryTarget) -> Self {
		self.target = target;
		self
	}

	/// Set the allowed destination of the [`Querier`].
	#[cfg(feature = "unstable")]
	#[must_use]
	pub const fn allowed_destination(mut self, allowed_destination: Locality) -> Self {
		self.allowed_destination = allowed_destination;
		self
	}

	/// Set the [`Querier`]s encoding
	#[must_use]
	pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
		self.encoding = encoding.into();
		self
	}

	/// Set a timeout for the [`Querier`].
	/// Default is 100ms
	#[must_use]
	pub const fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}
}

impl<P, C, S> QuerierBuilder<P, NoSelector, C, S>
where
	P: Send + Sync + 'static,
{
	/// Set the full expression for the query
	#[must_use]
	pub fn selector(self, selector: &str) -> QuerierBuilder<P, Selector, C, S> {
		let Self {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			storage,
			callback,
			mode,
			target,
			..
		} = self;
		QuerierBuilder {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector: Selector {
				selector: selector.into(),
			},
			callback,
			storage,
			mode,
			target,
		}
	}

	/// Set only the message qualifing part of the query.
	/// Will be prefixed with agents prefix.
	#[must_use]
	pub fn topic(self, topic: &str) -> QuerierBuilder<P, Selector, C, S> {
		let selector = selector_from(topic, self.context.prefix());
		self.selector(&selector)
	}
}

impl<P, K, S> QuerierBuilder<P, K, NoCallback, S>
where
	P: Send + Sync + 'static,
{
	/// Set query callback for response messages
	#[must_use]
	pub fn callback<C, F>(
		self,
		mut callback: C,
	) -> QuerierBuilder<P, K, Callback<ArcGetCallback<P>>, S>
	where
		C: FnMut(Context<P>, QueryableMsg) -> F + Send + Sync + 'static,
		F: Future<Output = Result<()>> + Send + Sync + 'static,
	{
		let Self {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			storage,
			mode,
			target,
			..
		} = self;
		let callback: GetCallback<P> = Box::new(move |ctx, msg| Box::pin(callback(ctx, msg)));
		let callback: ArcGetCallback<P> = Arc::new(Mutex::new(callback));
		QuerierBuilder {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback: Callback { callback },
			storage,
			mode,
			target,
		}
	}
}

impl<P, K, C> QuerierBuilder<P, K, C, NoStorage>
where
	P: Send + Sync + 'static,
{
	/// Provide agents storage for the query
	#[must_use]
	pub fn storage(self, storage: &mut ComponentType) -> QuerierBuilder<P, K, C, Storage> {
		let Self {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback,
			mode,
			target,
			..
		} = self;
		QuerierBuilder {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback,
			storage: Storage { storage },
			mode,
			target,
		}
	}
}

impl<P, S> QuerierBuilder<P, Selector, Callback<ArcGetCallback<P>>, S>
where
	P: Send + Sync + 'static,
{
	/// Build the [`Querier`]
	/// # Errors
	///
	pub fn build(self) -> Result<Querier<P>> {
		let Self {
			session_id,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			encoding,
			timeout,
			selector,
			callback: response,
			mode,
			target,
			..
		} = self;

		let session = context
			.session(&session_id)
			.ok_or(Error::NoZenohSession)?;

		let selector = selector.selector;
		let encoding = Encoding::from(encoding);
		let activity = ActivityType::new(selector.clone());
		let operational = OperationalType::new(activation_state);
		#[cfg(not(feature = "unstable"))]
		let parameter = QuerierParameter::new(mode, timeout, encoding, target);
		#[cfg(feature = "unstable")]
		let parameter = QuerierParameter::new(mode, timeout, encoding, target, allowed_destination);

		Ok(Querier::new(
			activity,
			operational,
			selector,
			parameter,
			session,
			context,
			response.callback,
		))
	}
}

impl<'a, P> QuerierBuilder<P, Selector, Callback<ArcGetCallback<P>>, Storage<'a>>
where
	P: Send + Sync + 'static,
{
	/// Build and add the query to the agents context
	/// # Errors
	pub fn add(self) -> Result<()> {
		let mut collection = self.storage.storage.clone();
		let q = self.build()?;
		collection.add_activity(Box::new(q));
		Ok(())
	}
}
// endregion:	--- QuerierBuilder
