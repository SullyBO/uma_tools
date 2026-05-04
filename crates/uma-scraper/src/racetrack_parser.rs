use crate::client::ScraperClient;
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use uma_core::models::racetrack::{Racetrack, RacetrackName};

use crate::error::{ScraperError, ScraperResult};

const RACETRACKS_URL: &str = "https://gametora.com/data/umamusume/racetracks.7d2f3355.json";

pub async fn fetch_racetracks(client: &ScraperClient) -> ScraperResult<Vec<Racetrack>> {
    let json = client.fetch(RACETRACKS_URL).await?;
    parse_racetrack_roster(&json)
}

fn parse_racetrack_roster(json: &str) -> ScraperResult<Vec<Racetrack>> {
    let items: Vec<Value> = serde_json::from_str(json).map_err(|e| {
        ScraperError::ParseError(format!("failed to parse skill conditions JSON: {e}"))
    })?;

    let mut racetracks = Vec::new();
    let mut courses_count = 0usize;
    let mut skip_reasons: HashMap<&str, usize> = HashMap::new();

    for item in &items {
        match parse_racetrack(item) {
            Ok(racetrack) => {
                courses_count += racetrack.courses.len();
                racetracks.push(racetrack)
            }
            Err(e) => {
                let id = item["id"].as_u64().unwrap_or(0);
                let reason = match &e {
                    ScraperError::MissingField(_) => "Missing field",
                    ScraperError::UnknownValue(_) => "Unknown value",
                    ScraperError::InvalidShape(_) => "Invalid shape",
                    ScraperError::JsonError(_) => "JSON deserialization error",
                    _ => "Other",
                };
                log::warn!("Failed to parse racetrack id {id}: {e}");
                *skip_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    info!("Racetrack roster parsing complete:");
    info!("{} Racetracks parsed", racetracks.len());
    info!("{} Courses parsed", courses_count);
    info!(
        "{} skipped racetracks out of {} total",
        skip_reasons.values().sum::<usize>(),
        items.len()
    );

    if !skip_reasons.is_empty() {
        info!("Skip breakdown:");
        let mut reasons: Vec<_> = skip_reasons.iter().collect();
        reasons.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (reason, count) in reasons {
            info!("  *{reason}: {count}");
        }
    }

    Ok(racetracks)
}

fn parse_racetrack(item: &Value) -> ScraperResult<Racetrack> {
    Ok(Racetrack::default())
}

fn parse_racetrack_name(racetrack_id: u32) -> ScraperResult<RacetrackName> {
    match racetrack_id {
        10001 => Ok(RacetrackName::Sapporo),
        10002 => Ok(RacetrackName::Hakodate),
        10003 => Ok(RacetrackName::Niigata),
        10004 => Ok(RacetrackName::Fukushima),
        10005 => Ok(RacetrackName::Nakayama),
        10006 => Ok(RacetrackName::Tokyo),
        10007 => Ok(RacetrackName::Chukyo),
        10008 => Ok(RacetrackName::Kyoto),
        10009 => Ok(RacetrackName::Hanshin),
        10010 => Ok(RacetrackName::Kokura),
        10101 => Ok(RacetrackName::Ooi),
        10103 => Ok(RacetrackName::Kawasaki),
        10104 => Ok(RacetrackName::Funabashi),
        10105 => Ok(RacetrackName::Morioka),
        10201 => Ok(RacetrackName::Longchamp),
        10202 => Ok(RacetrackName::SantaAnitaPark),
        10203 => Ok(RacetrackName::DelMar),
        _ => Err(ScraperError::UnknownValue(format!(
            "racetrack_id {}",
            racetrack_id
        ))),
    }
}
