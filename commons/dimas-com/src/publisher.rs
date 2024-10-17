// Copyright © 2023 Stephan Kunz

//! Module `publisher` provides a message sender `Publisher` which can be created using the `PublisherBuilder`.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;
use crate::error::Error;
use core::fmt::Debug;
use dimas_core::{
	enums::OperationState,
	message_types::Message,
	traits::{Capability, Context},
	Result,
};
use tracing::{instrument, Level};
#[cfg(feature = "unstable")]
use zenoh::{qos::Reliability, sample::Locality};
use zenoh::{
	qos::{CongestionControl, Priority},
	Wait,
};
// endregion:	--- modules

// region:		--- Publisher
/// Publisher
pub struct Publisher<P>
where
	P: Send + Sync + 'static,
{
	selector: String,
	/// Context for the Publisher
	context: Context<P>,
	activation_state: OperationState,
	#[cfg(feature = "unstable")]
	allowed_destination: Locality,
	congestion_control: CongestionControl,
	encoding: String,
	express: bool,
	priority: Priority,
	#[cfg(feature = "unstable")]
	reliability: Reliability,
	publisher: Option<zenoh::pubsub::Publisher<'static>>,
}

impl<P> Debug for Publisher<P>
where
	P: Send + Sync + 'static,
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Publisher")
			.field("selector", &self.selector)
			.field("initialized", &self.publisher.is_some())
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Publisher<P>
where
	P: Send + Sync + 'static,
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
	P: Send + Sync + 'static,
{
	/// Constructor for a [`Publisher`]
	#[allow(clippy::too_many_arguments)]
	#[must_use]
	pub const fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		#[cfg(feature = "unstable")] allowed_destination: Locality,
		congestion_control: CongestionControl,
		encoding: String,
		express: bool,
		priority: Priority,
		#[cfg(feature = "unstable")] reliability: Reliability,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			#[cfg(feature = "unstable")]
			allowed_destination,
			congestion_control,
			encoding,
			express,
			priority,
			#[cfg(feature = "unstable")]
			reliability,
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
		P: Send + Sync + 'static,
	{
		let session = self.context.session();
		let builder = session
			.declare_publisher(self.selector.clone())
			.congestion_control(self.congestion_control)
			.encoding(self.encoding.as_str())
			.express(self.express)
			.priority(self.priority);

		#[cfg(feature = "unstable")]
		let builder = builder
			.allowed_destination(self.allowed_destination)
			.reliability(self.reliability);

		let publisher = builder.wait()?;
		//.map_err(|_| DimasError::Put.into())?;
		self.publisher.replace(publisher);
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
			.as_ref()
			.ok_or(Error::AccessPublisher)?
			.put(message.value())
			.wait()
		{
			Ok(()) => Ok(()),
			Err(source) => Err(Error::PublishingPut { source }.into()),
		}
	}

	/// Send a "delete" message
	/// # Errors
	///
	#[instrument(level = Level::ERROR, skip_all)]
	pub fn delete(&self) -> Result<()> {
		match self
			.publisher
			.as_ref()
			.ok_or(Error::AccessPublisher)?
			.delete()
			.wait()
		{
			Ok(()) => Ok(()),
			Err(source) => Err(Error::PublishingDelete { source }.into()),
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
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Publisher<Props>>();
	}
}
