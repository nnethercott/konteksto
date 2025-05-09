use ndarray::ShapeError;

pub type Result<T> = std::result::Result<T, KontekstoError>;

#[derive(thiserror::Error, Debug)]
pub enum KontekstoError {
    #[error(transparent)]
    QdrantError(#[from] anyhow::Error),

    #[error(transparent)]
    ContextoError(#[from] reqwest::Error),

    #[error("linalg")]
    LinalgError(#[from] ShapeError)
}
