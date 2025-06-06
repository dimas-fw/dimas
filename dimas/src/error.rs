// Copyright © 2024 Stephan Kunz

//! `dimas` errors

use thiserror::Error;

// region:		--- Error
/// `dimas` error type
#[derive(Error, Debug)]
pub enum Error {
	/// activation of liveliness failed
	#[error("activation of liveliness failed: reason {source:?}")]
	ActivateLiveliness {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// manage state failed
	#[error("managing state failed")]
	ManageState,
	/// callback is missing
	#[error("callback is missing")]
	MissingCallback,
	/// callback is not implemented
	#[error("communicator is not implemented")]
	NotImplemented,
	/// get from collection failed
	#[error("get in {0} failed")]
	Get(String),
	/// get mutable from collection failed
	#[error("get mutable in {0} failed")]
	GetMut(String),
	/// a Mutex is poisoned.
	#[error("a Mutex poison error happened in {0}")]
	MutexPoison(String),
	/// read access failed
	#[error("accesssing the storage for read failed")]
	ReadAccess,
	/// write access failed
	#[error("accesssing the storage for write failed")]
	WriteAccess,
	/// read access to context failed
	#[error("read context for {0} failed")]
	ReadContext(String),
	/// write access to context failed
	#[error("write context for {0} failed")]
	ModifyStruct(String),
}
// region:		--- Error

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Error>();
	}
}
