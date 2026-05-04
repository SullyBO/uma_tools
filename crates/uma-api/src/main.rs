use axum::{Router, routing::get};
use sqlx::PgPool;
use std::sync::Arc;

mod routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
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
    log::info!("Running in {app_env} environment");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        pool: Arc::new(pool),
    };

    let app = Router::new()
        .route("/umas", get(routes::umas::list))
        .route("/umas/{id}", get(routes::umas::detail))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    log::info!("Listening on port {port}");
    axum::serve(listener, app).await.unwrap();
}
