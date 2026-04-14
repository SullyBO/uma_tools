use crate::error::ScraperResult;
use reqwest::Client;

pub async fn fetch_page(client: &Client, url: &str) -> ScraperResult<String> {
    let response = client.get(url).send().await?.error_for_status()?;

    Ok(response.text().await?)
}
