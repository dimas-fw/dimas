//! Copyright © 2023 Stephan Kunz

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),
}

impl Default for Error {
    fn default() -> Self {
        Self::Generic("error".to_string())
    }
}
