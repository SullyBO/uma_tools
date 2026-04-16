use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScraperError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("HTTP error {status}: {url}")]
    HttpError { status: u16, url: String },

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    #[error("Expected element not found: {0}")]
    ElementNotFound(String),

    #[error("Unexpected value encountered: {0}")]
    UnexpectedValue(String),
}

impl ScraperError {
    pub fn is_retryable(&self) -> bool {
        match self {
            // Network-level failures (timeouts, connection resets, etc.)
            ScraperError::NetworkError(e) => e.is_timeout() || e.is_connect(),
            // 429 is handled separately via RateLimited, but 5xx are retryable
            ScraperError::HttpError { status, .. } => *status >= 500,
            ScraperError::RateLimited { .. } => true,
            // Parse/logic errors are not going to resolve themselves
            _ => false,
        }
    }
}

pub type ScraperResult<T> = Result<T, ScraperError>;
