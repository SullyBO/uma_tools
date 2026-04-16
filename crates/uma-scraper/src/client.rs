use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::error::{ScraperError, ScraperResult};

const DEFAULT_MAX_CONCURRENCY: usize = 5;
const DEFAULT_MIN_DELAY: Duration = Duration::from_millis(500);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_BACKOFF_BASE: Duration = Duration::from_secs(1);

pub struct ScraperClient {
    client: Client,
    semaphore: Arc<Semaphore>,
    min_delay: Duration,
    max_retries: u32,
    backoff_base: Duration,
}

pub struct ScraperClientBuilder {
    max_concurrency: usize,
    min_delay: Duration,
    max_retries: u32,
    backoff_base: Duration,
}

impl ScraperClientBuilder {
    pub fn new() -> Self {
        Self {
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            min_delay: DEFAULT_MIN_DELAY,
            max_retries: DEFAULT_MAX_RETRIES,
            backoff_base: DEFAULT_BACKOFF_BASE,
        }
    }

    pub fn max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n;
        self
    }

    pub fn min_delay(mut self, delay: Duration) -> Self {
        self.min_delay = delay;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn backoff_base(mut self, base: Duration) -> Self {
        self.backoff_base = base;
        self
    }

    pub fn build(self) -> ScraperClient {
        ScraperClient {
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(self.max_concurrency)),
            min_delay: self.min_delay,
            max_retries: self.max_retries,
            backoff_base: self.backoff_base,
        }
    }
}

impl Default for ScraperClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScraperClient {
    pub fn builder() -> ScraperClientBuilder {
        ScraperClientBuilder::new()
    }

    /// Fetch a single page, with retries. Intended for one-off pages like the skills wiki.
    pub async fn fetch(&self, url: &str) -> ScraperResult<String> {
        self.fetch_with_retries(url).await
    }

    /// Fetch many pages concurrently, respecting the concurrency limit and rate limits.
    /// Failures are captured per-URL rather than aborting the whole batch.
    pub async fn fetch_all(&self, urls: &[&str]) -> Vec<ScraperResult<String>> {
        let semaphore = self.semaphore.clone();

        let tasks: Vec<_> = urls
            .iter()
            .map(|&url| {
                let sem = semaphore.clone();
                let url = url.to_string();
                async move {
                    // Acquiring the permit gates concurrency across all tasks
                    let _permit = sem.acquire().await.expect("semaphore closed");
                    self.fetch_with_retries(&url).await
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }

    async fn fetch_with_retries(&self, url: &str) -> ScraperResult<String> {
        let mut attempt = 0;

        loop {
            match self.fetch_once(url).await {
                Ok(html) => {
                    sleep(self.min_delay).await;
                    return Ok(html);
                }
                Err(ScraperError::RateLimited { retry_after_secs }) => {
                    sleep(Duration::from_secs(retry_after_secs)).await;
                }
                Err(e) if e.is_retryable() && attempt < self.max_retries => {
                    let backoff = self.backoff_base * 2u32.pow(attempt);
                    sleep(backoff).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn fetch_once(&self, url: &str) -> ScraperResult<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(ScraperError::NetworkError)?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after_secs = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(ScraperError::RateLimited { retry_after_secs });
        }

        if !status.is_success() {
            return Err(ScraperError::HttpError {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }

        Ok(response.text().await.map_err(ScraperError::NetworkError)?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ScraperClient;
    use crate::error::ScraperError;

    fn test_client() -> ScraperClient {
        ScraperClient::builder()
            .min_delay(Duration::ZERO)
            .backoff_base(Duration::ZERO)
            .build()
    }

    #[tokio::test]
    async fn fetch_returns_body_on_200() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>skills</html>"))
            .mount(&server)
            .await;

        let result = test_client().fetch(&server.uri()).await;
        assert_eq!(result.unwrap(), "<html>skills</html>");
    }

    #[tokio::test]
    async fn fetch_retries_on_500_and_succeeds() {
        let server = MockServer::start().await;

        // Fail twice, then succeed
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let result = test_client().fetch(&server.uri()).await;
        assert_eq!(result.unwrap(), "ok");
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn fetch_gives_up_after_max_retries() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = ScraperClient::builder()
            .min_delay(Duration::ZERO)
            .backoff_base(Duration::ZERO)
            .max_retries(3)
            .build();

        let result = client.fetch(&server.uri()).await;
        assert!(matches!(
            result,
            Err(ScraperError::HttpError { status: 500, .. })
        ));
        assert_eq!(server.received_requests().await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn fetch_does_not_retry_on_404() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let result = test_client().fetch(&server.uri()).await;
        assert!(matches!(
            result,
            Err(ScraperError::HttpError { status: 404, .. })
        ));
        // Should have given up immediately, no retries
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn fetch_honors_retry_after_on_429() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let result = test_client().fetch(&server.uri()).await;
        assert_eq!(result.unwrap(), "ok");
        // 429 should not have consumed a retry attempt — only 2 requests total
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn fetch_all_captures_failures_without_aborting() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let urls = vec![server.uri(), server.uri()];
        let url_refs: Vec<&str> = urls.iter().map(String::as_str).collect();

        let results = test_client().fetch_all(&url_refs).await;

        assert_eq!(results.len(), 2);

        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(successes, 1);
        assert_eq!(failures, 1);
    }
}
