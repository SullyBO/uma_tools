use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScraperError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    #[error("Expected element not found: {0}")]
    ElementNotFound(String),

    #[error("Unexpected value encountered: {0}")]
    UnexpectedValue(String),
}

pub type ScraperResult<T> = Result<T, ScraperError>;
