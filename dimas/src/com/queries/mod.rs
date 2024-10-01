// Copyright © 2024 Stephan Kunz

//! Module handles queries.
//!

// region:    	--- modules
mod querier;
mod querier_builder;
mod queryable;
mod queryable_builder;

// flatten
pub use querier::*;
pub use querier_builder::*;
pub use queryable::*;
pub use queryable_builder::*;

use dimas_core::{
	error::Result,
	message_types::{QueryMsg, QueryableMsg},
	traits::Context,
};
use std::sync::{Arc, Mutex};
// endregion: 	--- modules

// region:    	--- types
// ------ Querier
/// type definition for a queriers atomic reference counted `response` callback
pub type ArcQuerierCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, QueryableMsg) -> Result<()> + Send + Sync + 'static>>;
// ------ Queryable
/// type defnition for a queryables atomic reference counted `request` callback
pub type ArcQueryableCallback<P> =
	Arc<Mutex<dyn FnMut(Context<P>, QueryMsg) -> Result<()> + Send + Sync + 'static>>;
// endregion: 	--- types
