use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegionEraserError {
    #[error("Ears error: {0}")]
    EarsError(#[from] ears_rs::utils::errors::EarsError),
    
    #[error("Image error: {0}")]
    ImageError(#[from] image::error::ImageError),
}

pub(crate) type Result<T> = std::result::Result<T, RegionEraserError>;