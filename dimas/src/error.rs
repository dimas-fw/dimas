//! Copyright © 2023 Stephan Kunz

// region:    --- types
/// Enables simplified usage of Result with crates Error type
pub type Result<T> = core::result::Result<T, Error>;

/// crates temporary Error type
pub type Error = Box<dyn std::error::Error>;
// endregion: --- types

// region:    --- Error
//#[derive(thiserror::Error, Debug, Clone)]
//pub enum Error {
//    #[error("Generic {0}")]
//    Generic(String),
//}
//
//impl Default for Error {
//    fn default() -> Self {
//        Self::Generic("error".to_string())
//    }
//}
// endregion: --- Error
