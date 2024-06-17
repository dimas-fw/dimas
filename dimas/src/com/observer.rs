// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::{
	enums::OperationState, error::{DimasError, Result}, message_types::{Message, ObservableMsg}, traits::{Capability, Context}
};
use tokio::task::JoinHandle;
use tracing::{error, instrument, warn, Level};
use zenoh::{
	prelude::{sync::SyncResolve, SampleKind},
	query::{ConsolidationMode, QueryTarget},
	sample::Locality,
};

use super::ArcObserverCallback;
// endregion:	--- modules

// region:		--- Observer
/// Observer
pub struct Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The observers key expression
	selector: String,
	/// Context for the Observer
	context: Context<P>,
	activation_state: OperationState,
	callback: ArcObserverCallback<P>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Observer").finish_non_exhaustive()
	}
}

impl<P> Capability for Observer<P>
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

impl<P> Observer<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for an [`Observer`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		callback: ArcObserverCallback<P>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			callback,
			handle: None,
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
	#[instrument(level = Level::TRACE, skip_all)]
	fn init(&mut self) -> Result<()> {
		Ok(())
	}

	/// De-Initialize
	/// # Errors
	///
	#[allow(clippy::unnecessary_wraps)]
	fn de_init(&mut self) -> Result<()> {
		self.handle.take();
		Ok(())
	}

	/// Run an observation with an optional [`Message`].
	#[instrument(name="observer", level = Level::ERROR, skip_all)]
	pub fn observe(&self, message: Option<Message>) -> Result<()> {
		let cb = self.callback.clone();
		let session = self.context.session();
		let mut query = session
			.get(&self.selector)
			.target(QueryTarget::All)
			.consolidation(ConsolidationMode::None)
			.allowed_destination(Locality::Any);

		//if let Some(timeout) = self.timeout {
		//	query = query.timeout(timeout);
		//};

		if let Some(message) = message {
			let value = message.value().to_owned();
			query = query.with_value(value);
		};

		let replies = query
			.res_sync()
			.map_err(|_| DimasError::ShouldNotHappen)?;

		while let Ok(reply) = replies.recv() {
			match reply.sample {
				Ok(sample) => match sample.kind {
					SampleKind::Put => {
						let content: Vec<u8> = sample.value.try_into()?;
						let msg = ObservableMsg(content);
						let guard = cb.lock();
						match guard {
							Ok(mut lock) => {
								if let Err(error) = lock(&self.context.clone(), msg) {
									error!("callback failed with {error}");
								}
							}
							Err(err) => {
								error!("callback lock failed with {err}");
							}
						}
					}
					SampleKind::Delete => {
						error!("Delete in Observer");
					}
				},
				Err(err) => error!("receive error: {err})"),
			}
		}
		Ok(())
	}
}
// endregion:	--- Observer

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observer<Props>>();
	}
}
