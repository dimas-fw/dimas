// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

// region:		--- modules
// these ones are only for doc needed
#[cfg(doc)]
use crate::agent::Agent;
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	message_types::Message,
	traits::{Capability, Context},
};
use std::fmt::Debug;
use tracing::{instrument, Level};
use zenoh::{
	prelude::sync::SyncResolve,
	publication::{CongestionControl, Priority},
	SessionDeclarations,
};
// endregion:	--- modules

// region:		--- Publisher
/// Publisher
pub struct Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	selector: String,
	/// Context for the Publisher
	context: Context<P>,
	activation_state: OperationState,
	priority: Priority,
	congestion_control: CongestionControl,
	publisher: Option<zenoh::publication::Publisher<'static>>,
}

impl<P> Debug for Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Publisher")
			.field("selector", &self.selector)
			.field("initialized", &self.publisher.is_some())
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if state >= &self.activation_state {
			return self.init();
		} else if state < &self.activation_state {
			return self.de_init();
		}
		Ok(())
	}
}

impl<P> Publisher<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for a [`Publisher`]
	#[must_use]
	pub const fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		priority: Priority,
		congestion_control: CongestionControl,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			priority,
			congestion_control,
			publisher: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Initialize
	/// # Errors
	///
	fn init(&mut self) -> Result<()>
	where
		P: Send + Sync + Unpin + 'static,
	{
		let publ = self
			.context
			.session()
			.declare_publisher(self.selector.clone())
			.congestion_control(self.congestion_control)
			.priority(self.priority)
			.res_sync()?;
		//.map_err(|_| DimasError::Put.into())?;
		self.publisher.replace(publ);
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	///
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()> {
		self.publisher.take();
		Ok(())
	}

	/// Send a "put" message
	/// # Errors
	///
	#[instrument(name="publish", level = Level::ERROR, skip_all)]
	pub fn put(&self, message: Message) -> Result<()> {
		match self
			.publisher
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.put(message.0)
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::Put.into()),
		}
	}

	/// Send a "delete" message
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self) -> Result<()> {
		match self
			.publisher
			.clone()
			.ok_or(DimasError::ShouldNotHappen)?
			.delete()
			.res_sync()
		{
			Ok(()) => Ok(()),
			Err(_) => Err(DimasError::Delete.into()),
		}
	}
}
// endregion:	--- Publisher

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher<Props>>();
	}
}
