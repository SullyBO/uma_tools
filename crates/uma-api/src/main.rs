use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use sqlx::PgPool;
use std::sync::Arc;
use subtle::ConstantTimeEq;

mod error;
mod routes;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    api_key: ApiKey,
}

#[derive(Clone)]
struct ApiKey(String);

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl ApiKey {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[tokio::main]
async fn main() {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file = match app_env.as_str() {
        "prod" => ".env.prod",
        _ => ".env",
    };
    dotenvy::from_filename(env_file).ok();
    env_logger::init();
    let api_key = ApiKey(
        std::env::var("API_KEY")
            .expect("API_KEY must be set")
            .trim()
            .to_string(),
    );
    log::info!("Running in {app_env} environment");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        pool: Arc::new(pool),
        api_key,
    };

    let app = Router::new()
        .route("/umas", get(routes::umas::list))
        .route("/umas/index", get(routes::umas::index))
        .route("/umas/{id}", get(routes::umas::detail))
        .route("/skills", get(routes::skills::list))
        .route("/skills/{id}", get(routes::skills::detail))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    log::info!("Listening on port {port}");
    axum::serve(listener, app).await.unwrap();
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided_key = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided_key {
        Some(key) if key.as_bytes().ct_eq(state.api_key.as_bytes()).into() => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
