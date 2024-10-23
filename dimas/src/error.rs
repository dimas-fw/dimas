// Copyright © 2024 Stephan Kunz

//! Errors

#[doc(hidden)]
extern crate alloc;

//#[cfg(feature = "std")]
//extern crate std;

// region:		--- Error
/// Com error type.
pub enum Error {
	/// activation of liveliness failed
	#[cfg(feature = "unstable")]
	ActivateLiveliness {
		/// the original zenoh error
		source: Box<dyn core::error::Error + Send + Sync>,
	},
	/// manage state failed
	ManageState,
	/// callback is missing
	MissingCallback,
	/// get from collection failed
	Get(String),
	/// get mutable from collection failed
	GetMut(String),
	/// a Mutex is poisoned.
	MutexPoison(String),
	/// read access failed
	ReadAccess,
	/// write access failed
	WriteAccess,
	/// read access failed
	ReadContext(String),
	/// write access failed
	ModifyContext(String),
}
// region:		--- Error

// region:      --- boilerplate
impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			#[cfg(feature = "unstable")]
			Self::ActivateLiveliness { source } => {
				write!(f, "activation of liveliness failed: reason {source}")
			}
			Self::ManageState => {
				write!(f, "managing state failed")
			}
			Self::MissingCallback => {
				write!(f, "callback is missing")
			}
			Self::Get(location) => {
				write!(f, "get in {location} failed")
			}
			Self::GetMut(location) => {
				write!(f, "get mutable in {location} failed")
			}
			Self::MutexPoison(location) => {
				write!(f, "an Mutex poison error happened in {location}")
			}
			Self::ReadAccess => {
				write!(f, "accesssing the storage for read failed")
			}
			Self::WriteAccess => {
				write!(f, "accesssing the storage for write failed")
			}
			Self::ReadContext(location) => {
				write!(f, "read context for {location} failed")
			}
			Self::ModifyContext(location) => {
				write!(f, "write context for {location} failed")
			}
		}
	}
}

impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match *self {
			#[cfg(feature = "unstable")]
			Self::ActivateLiveliness { ref source } => Some(source.as_ref()),
			Self::ManageState
			| Self::MissingCallback
			| Self::Get(_)
			| Self::GetMut(_)
			| Self::MutexPoison { .. }
			| Self::ReadAccess
			| Self::WriteAccess
			| Self::ReadContext(_)
			| Self::ModifyContext(_) => None,
		}
	}
}
// endregion:   --- boilerplate
