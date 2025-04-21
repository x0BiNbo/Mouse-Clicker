use std::io;
use std::fmt;
use image::error::ImageError;
use base64::DecodeError;
use serde_json::Error as JsonError;

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    ParseError(String),
    ImageError(ImageError),
    Base64Error(DecodeError),
    JsonError(JsonError),
}

// Implement Send for AppError
unsafe impl Send for AppError {}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseError(s) => write!(f, "Parse error: {}", s),
            AppError::ImageError(e) => write!(f, "Image error: {}", e),
            AppError::Base64Error(e) => write!(f, "Base64 error: {}", e),
            AppError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<ImageError> for AppError {
    fn from(error: ImageError) -> Self {
        AppError::ImageError(error)
    }
}

impl From<DecodeError> for AppError {
    fn from(error: DecodeError) -> Self {
        AppError::Base64Error(error)
    }
}

impl From<JsonError> for AppError {
    fn from(error: JsonError) -> Self {
        AppError::JsonError(error)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
