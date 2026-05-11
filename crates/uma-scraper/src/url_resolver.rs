use crate::client::ScraperClient;
use crate::error::ScraperResult;
use std::collections::HashMap;

const MANIFEST_URL: &str = "https://gametora.com/data/manifests/umamusume.json";
const BASE_URL: &str = "https://gametora.com/data/umamusume";

async fn fetch_manifest(client: &ScraperClient) -> ScraperResult<HashMap<String, String>> {
    let json = client.fetch(MANIFEST_URL).await?;
    serde_json::from_str(&json).map_err(|e| crate::error::ScraperError::JsonError(e.to_string()))
}

async fn resolve_url(client: &ScraperClient, key: &str, path: &str) -> ScraperResult<String> {
    let manifest = fetch_manifest(client).await?;
    let hash = manifest.get(key).ok_or_else(|| {
        crate::error::ScraperError::ParseError(format!("Key '{}' not found in manifest", key))
    })?;
    Ok(format!("{}/{}.{}.json", BASE_URL, path, hash))
}

pub async fn resolve_uma_url(client: &ScraperClient) -> ScraperResult<String> {
    resolve_url(client, "character-cards", "character-cards").await
}

pub async fn resolve_skills_url(client: &ScraperClient) -> ScraperResult<String> {
    resolve_url(client, "skills", "skills").await
}

pub async fn resolve_conditions_url(client: &ScraperClient) -> ScraperResult<String> {
    resolve_url(client, "static/skill_conditions", "static/skill_conditions").await
}

pub async fn resolve_predicted_release_dates_url(client: &ScraperClient) -> ScraperResult<String> {
    resolve_url(
        client,
        "en/foresight/predicted_releases",
        "en/foresight/predicted_releases",
    )
    .await
}
