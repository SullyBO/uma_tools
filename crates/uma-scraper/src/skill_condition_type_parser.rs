use crate::client::ScraperClient;
use log::info;
use serde_json::Value;
use uma_core::models::skill::ConditionType;

use crate::error::{ScraperError, ScraperResult};

const SKILL_CONDITIONS_URL: &str =
    "https://gametora.com/data/umamusume/static/skill_conditions.32e9f707.json";

pub async fn fetch_skill_condition_types(
    client: &ScraperClient,
) -> ScraperResult<Vec<ConditionType>> {
    let json = client.fetch(SKILL_CONDITIONS_URL).await?;
    parse_skill_condition_types(&json)
}

fn parse_skill_condition_types(json: &str) -> ScraperResult<Vec<ConditionType>> {
    let items: Vec<Value> = serde_json::from_str(json).map_err(|e| {
        ScraperError::ParseError(format!("failed to parse skill conditions JSON: {e}"))
    })?;

    let mut entries = Vec::new();
    let mut skipped = 0usize;

    for item in &items {
        match parse_condition_item(item) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                let name = item["name"].as_str().unwrap_or("unknown");
                log::warn!("Failed to parse condition '{name}': {e}");
                skipped += 1;
            }
        }
    }

    info!(
        "Condition type parsing complete: {} parsed, {} skipped out of {} total",
        entries.len(),
        skipped,
        items.len()
    );

    Ok(entries)
}

fn parse_condition_item(item: &Value) -> ScraperResult<ConditionType> {
    let cond_key = item["name"]
        .as_str()
        .ok_or_else(|| ScraperError::ParseError("missing name".into()))?
        .to_string();

    let description = {
        let desc = item["desc"].as_str().unwrap_or("").trim().to_string();
        let note = item["note"].as_str().unwrap_or("").trim().to_string();
        if note.is_empty() {
            desc
        } else {
            format!("{desc} {note}")
        }
    };

    let example = {
        let example_text = item["example"].as_str().unwrap_or("").trim().to_string();
        let example_meaning = item["example_meaning"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();
        if example_text.is_empty() {
            None
        } else if example_meaning.is_empty() {
            Some(example_text)
        } else {
            Some(format!("{example_text} — {example_meaning}"))
        }
    };

    Ok(ConditionType {
        cond_key,
        description,
        example,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_item() -> serde_json::Value {
        serde_json::json!({
            "name": "accumulatetime",
            "desc": "The number of seconds since the race has started.",
            "note": "",
            "example": "accumulatetime>=5",
            "example_meaning": "The race has been ongoing for at least 5 seconds.",
            "values": ""
        })
    }

    #[test]
    fn parses_valid_item() {
        let entry = parse_condition_item(&valid_item()).unwrap();
        assert_eq!(entry.cond_key, "accumulatetime");
        assert_eq!(
            entry.description,
            "The number of seconds since the race has started."
        );
        let example = entry.example.unwrap();
        assert!(example.contains("accumulatetime>=5"));
        assert!(example.contains("at least 5 seconds"));
    }

    #[test]
    fn appends_note_to_description() {
        let mut item = valid_item();
        item["note"] = serde_json::json!("Some additional note.");
        let entry = parse_condition_item(&item).unwrap();
        assert!(
            entry
                .description
                .contains("The number of seconds since the race has started.")
        );
        assert!(entry.description.contains("Some additional note."));
    }

    #[test]
    fn example_is_none_when_empty() {
        let mut item = valid_item();
        item["example"] = serde_json::json!("");
        let entry = parse_condition_item(&item).unwrap();
        assert!(entry.example.is_none());
    }

    #[test]
    fn example_has_no_meaning_when_meaning_empty() {
        let mut item = valid_item();
        item["example_meaning"] = serde_json::json!("");
        let entry = parse_condition_item(&item).unwrap();
        assert_eq!(entry.example.unwrap(), "accumulatetime>=5");
    }

    #[test]
    fn errors_on_missing_name() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("name");
        let result = parse_condition_item(&item);
        assert!(matches!(result, Err(ScraperError::ParseError(_))));
    }

    #[test]
    fn roster_tolerates_bad_items() {
        let bad_item = serde_json::json!({"desc": "no name here"});
        let json = serde_json::json!([valid_item(), bad_item]).to_string();
        let entries = parse_skill_condition_types(&json).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn parses_full_roster() {
        let json = serde_json::json!([valid_item(), valid_item()]).to_string();
        let entries = parse_skill_condition_types(&json).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
