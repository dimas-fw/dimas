// Copyright © 2024 Stephan Kunz

// region:		--- modules
use dimas_core::{
	enums::OperationState,
	error::{DimasError, Result},
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
// endregion:	--- modules

// region:		--- Observable
/// Observable
pub struct Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Context for the Observable
	context: Context<P>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Observable")
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		//		if (state >= &self.activation_state) && self.handle.is_none() {
		//			return self.start();
		//		} else if (state < &self.activation_state) && self.handle.is_some() {
		//			self.stop();
		//			return Ok(());
		//		}
		Ok(())
	}
}

impl<P> Observable<P> where P: Send + Sync + Unpin + 'static {}
// endregion:	--- Observable

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observable<Props>>();
	}
}
