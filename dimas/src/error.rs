// Copyright © 2023 Stephan Kunz

// region:    --- types
/// Enables simplified usage of Result with crates Error type
pub type Result<T> = core::result::Result<T, DimasError>;
// endregion: --- types

// region:    --- Error
#[derive(thiserror::Error, Debug, Clone)]
pub enum DimasError {
    #[error("Generic {0}")]
    Generic(String),
}

impl Default for DimasError {
    fn default() -> Self {
        Self::Generic("error".to_string())
    }
}
// endregion: --- Error
