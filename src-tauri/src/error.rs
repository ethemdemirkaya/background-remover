use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("image not found in cache: {0}")]
    UnknownImage(String),

    #[error("image decode failed: {0}")]
    Decode(#[from] image::ImageError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Msg(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
