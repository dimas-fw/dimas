// Copyright © 2024 Stephan Kunz

//! Lifecycle interface for `DiMAS` entities
//!

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use anyhow::Result;
use core::fmt::Debug;
use tracing::{event, instrument, Level};

use super::{Error, OperationState};
// endregion:	--- modules

// region:		--- Operational
/// Contract for [`Operational`]
pub trait Operational: Transitions + Debug + Send + Sync {
	/// Read the entities state when it shall be active
	/// different from parent components [`OperationState`] can be provided.
	/// The default is [`OperationState::Undefined`]
	#[must_use]
	fn activation_state(&self) -> OperationState;

	/// Write the entities state when it shall be active
	fn set_activation_state(&mut self, _state: OperationState);

	/// Calculate the desired [`OperationState`] from a given [`OperationState`].
	#[must_use]
	fn desired_state(&self, state: OperationState) -> OperationState {
		let state: i32 = state.into();
		let state_diff = OperationState::Active - self.activation_state();

		// limit to bounds [`OperationState::Created`] <=> [`OperationState::Active`]
		let min_state: i32 = OperationState::Created.into();
		let max_state: i32 = OperationState::Active.into();
		let desired_state_int = min_state.max(max_state.min(state + state_diff));

		OperationState::try_from(desired_state_int)
			.unwrap_or_else(|_| panic!("should be infallible"))
	}

	/// Read the entities current [`OperationState`] must be provided
	#[must_use]
	fn state(&self) -> OperationState;

	/// Write the entities current [`OperationState`] must be provided
	fn set_state(&mut self, _state: OperationState);

	/// Call the appropriate transitions and return the reached state.
	/// # Errors
	/// In case of error, the [`Operational`]s state is set to [`OperationalState::Error`]
	#[instrument(level = Level::TRACE, skip_all)]
	fn state_transitions(&mut self, state: OperationState) -> Result<()> {
		event!(Level::TRACE, "state_transitions");
		let mut next_state;
		while self.state() < state {
			assert!(self.state() < OperationState::Active);
			next_state = self.state() + 1;
			// next do own transition
			match self.state() {
				OperationState::Error | OperationState::Active => {
					return Err(Error::ManageState.into())
				}
				OperationState::Undefined => {} // no transition for now
				OperationState::Created => {
					self.configure()?;
				}
				OperationState::Configured => {
					self.commission()?;
				}
				OperationState::Inactive => {
					self.wakeup()?;
				}
				OperationState::Standby => {
					self.activate()?;
				}
			}
			// update own state
			self.set_state(next_state);
		}

		// step down?
		while self.state() > state {
			assert!(self.state() > OperationState::Created);
			next_state = self.state() - 1;
			// next do own transition
			match self.state() {
				OperationState::Error | OperationState::Undefined | OperationState::Created => {
					return Err(Error::ManageState.into())
				}
				OperationState::Active => {
					self.deactivate()?;
				}
				OperationState::Standby => {
					self.suspend()?;
				}
				OperationState::Inactive => {
					self.decommission()?;
				}
				OperationState::Configured => {
					self.deconfigure()?;
				}
			}
			// update own state
			self.set_state(next_state);
		}

		Ok(())
	}
}

/// Transition contract for [`Operational`]
pub trait Transitions: Debug + Send + Sync {
	/// configuration transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn configure(&mut self) -> Result<()> {
		Ok(())
	}

	/// comissioning transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn commission(&mut self) -> Result<()> {
		Ok(())
	}

	/// wake up transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn wakeup(&mut self) -> Result<()> {
		Ok(())
	}

	/// activate transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn activate(&mut self) -> Result<()> {
		Ok(())
	}

	/// deactivate transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn deactivate(&mut self) -> Result<()> {
		Ok(())
	}

	/// suspend transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn suspend(&mut self) -> Result<()> {
		Ok(())
	}

	/// decomission transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn decommission(&mut self) -> Result<()> {
		Ok(())
	}

	/// deconfigure transition
	/// The default implementation just returns Ok(())
	/// # Errors
	/// if something went wrong
	fn deconfigure(&mut self) -> Result<()> {
		Ok(())
	}
}
// endregion:	--- Operational

#[cfg(test)]
mod tests {
	use super::*;
	use crate::OperationalType;
	use alloc::boxed::Box;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Box<dyn Operational>>();
	}

	#[dimas_macros::operational]
	struct TestOperational {
		/// A value to test that all hooks have been processed
		value: i32,
	}

	impl Transitions for TestOperational {
		fn configure(&mut self) -> Result<()> {
			self.value += 1;
			Ok(())
		}

		fn commission(&mut self) -> Result<()> {
			self.value += 2;
			Ok(())
		}

		fn wakeup(&mut self) -> Result<()> {
			self.value += 4;
			Ok(())
		}

		fn activate(&mut self) -> Result<()> {
			self.value += 8;
			Ok(())
		}

		fn deactivate(&mut self) -> Result<()> {
			self.value -= 8;
			Ok(())
		}

		fn suspend(&mut self) -> Result<()> {
			self.value -= 4;
			Ok(())
		}

		fn decommission(&mut self) -> Result<()> {
			self.value -= 2;
			Ok(())
		}

		fn deconfigure(&mut self) -> Result<()> {
			self.value -= 1;
			Ok(())
		}
	}

	fn create_test_data() -> TestOperational {
		let operational = TestOperational::default();
		assert_eq!(operational.state(), OperationState::Undefined);
		assert_eq!(operational.activation_state(), OperationState::Active);
		operational
	}

	#[test]
	fn operational() {
		let mut operational = create_test_data();
		assert!(operational
			.state_transitions(OperationState::Created)
			.is_ok());
		assert_eq!(operational.value, 0);
		assert_eq!(operational.state(), OperationState::Created);

		assert!(operational
			.state_transitions(OperationState::Active)
			.is_ok());
		assert_eq!(operational.value, 15);
		assert_eq!(operational.state(), OperationState::Active);

		assert!(operational
			.state_transitions(OperationState::Inactive)
			.is_ok());
		assert_eq!(operational.value, 3);
		assert_eq!(operational.state(), OperationState::Inactive);

		assert!(operational
			.state_transitions(OperationState::Created)
			.is_ok());
		assert_eq!(operational.value, 0);
		assert_eq!(operational.state(), OperationState::Created);
	}
}
