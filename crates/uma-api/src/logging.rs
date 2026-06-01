use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct AxiomClient {
    token: String,
    dataset: String,
    client: reqwest::Client,
}

impl AxiomClient {
    pub fn new(token: String, dataset: String) -> Self {
        Self {
            token,
            dataset,
            client: reqwest::Client::new(),
        }
    }

    pub async fn ingest(&self, event: serde_json::Value) {
        let url = format!("https://api.axiom.co/v1/datasets/{}/ingest", self.dataset);
        let result = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&vec![event])
            .send()
            .await;

        match result {
            Ok(res) if !res.status().is_success() => {
                log::warn!("Axiom ingest failed with status: {}", res.status());
            }
            Err(e) => {
                log::warn!("Axiom ingest error: {e}");
            }
            _ => {}
        }
    }
}

pub async fn axiom_middleware(
    axum::extract::State(client): axum::extract::State<AxiomClient>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());
    let start = Instant::now();

    let response = next.run(request).await;

    let latency_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    let client = client.clone();
    tokio::spawn(async move {
        client
            .ingest(serde_json::json!({
                "_time": chrono::Utc::now().to_rfc3339(),
                "api_version": env!("CARGO_PKG_VERSION"),
                "method": method,
                "path": path,
                "query": query,
                "status": status,
                "latency_ms": latency_ms,
            }))
            .await;
    });

    response
}
