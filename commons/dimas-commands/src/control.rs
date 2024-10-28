// Copyright © 2024 Stephan Kunz

//! Commands for `DiMAS` control & monitoring programs

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region:		--- modules
use crate::messages::AboutEntity;
use alloc::{
	string::{String, ToString},
	vec::Vec,
};
use dimas_com::{traits::CommunicatorImplementationMethods, zenoh::Communicator};
use dimas_core::{
	enums::{OperationState, Signal},
	message_types::Message,
	utils::selector_from,
	Result,
};
#[cfg(feature = "std")]
use std::collections::HashMap;
// endregion:	--- modules

// region:		--- set_state
/// Set the [`OperationState`] of `DiMAS` entities
/// # Errors
#[cfg(feature = "std")]
pub fn set_state(
	com: &Communicator,
	base_selector: &String,
	state: Option<OperationState>,
) -> Result<Vec<AboutEntity>> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = selector_from("signal", Some(base_selector));
	let message = Message::encode(&Signal::State { state });
	// set state for entities matching the selector
	com.get(
		&selector,
		Some(message),
		Some(&mut |response| -> Result<()> {
			let response: AboutEntity = response.decode()?;
			map.entry(response.zid().to_string())
				.or_insert(response);
			Ok(())
		}),
	)?;

	let result: Vec<AboutEntity> = map.values().cloned().collect();

	Ok(result)
}
// endregion:	--- set_state

// region:		--- shutdown
/// Shutdown of `DiMAS` entities
/// # Errors
#[cfg(feature = "std")]
pub fn shutdown(com: &Communicator, base_selector: &String) -> Result<Vec<AboutEntity>> {
	let mut map: HashMap<String, AboutEntity> = HashMap::new();

	let selector = selector_from("signal", Some(base_selector));
	let message = Message::encode(&Signal::Shutdown);
	// set state for entities matching the selector
	com.get(
		&selector,
		Some(message),
		Some(&mut |response| -> Result<()> {
			let response: AboutEntity = response.decode()?;
			map.entry(response.zid().to_string())
				.or_insert(response);
			Ok(())
		}),
	)?;

	let result: Vec<AboutEntity> = map.values().cloned().collect();

	Ok(result)
}
// endregion:	--- shutdown
