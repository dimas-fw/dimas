// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
use tracing::{instrument, warn, Level};

use super::{ArcFeedbackCallback, ArcResultCallback};
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
	result_callback: ArcResultCallback<P>,
	feedback_callback: Option<ArcFeedbackCallback<P>>,
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
		if (state >= &self.activation_state) && self.handle.is_none() {
			return self.start();
		} else if (state < &self.activation_state) && self.handle.is_some() {
			self.stop();
			return Ok(());
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
		result_callback: ArcResultCallback<P>,
		feedback_callback: Option<ArcFeedbackCallback<P>>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			result_callback,
			feedback_callback,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Start or restart the Observer.
	/// An already running Observer will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.result_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.result_callback.clear_poison();
			}

			if let Some(fcb) = self.feedback_callback.clone() {
				if fcb.lock().is_err() {
					warn!("found poisoned delete Mutex");
					fcb.clear_poison();
				}
			}
		}
		Ok(())
	}

	/// Stop a running Observer
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
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
