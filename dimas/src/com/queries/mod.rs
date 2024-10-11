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
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::Mutex;
// endregion: 	--- modules

// region:    	--- types
/// type definition for a queriers `response` callback
type QuerierCallback<P> =
	Box<dyn FnMut(Context<P>, QueryableMsg) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// type definition for a queriers atomic reference counted `response` callback
type ArcQuerierCallback<P> = Arc<Mutex<QuerierCallback<P>>>;
/// type defnition for a queryables `request` callback
type QueryableCallback<P> =
	Box<dyn FnMut(Context<P>, QueryMsg) -> BoxFuture<'static, Result<()>> + Send + Sync>;
/// type defnition for a queryables atomic reference counted `request` callback
type ArcQueryableCallback<P> = Arc<Mutex<QueryableCallback<P>>>;
// endregion: 	--- types
