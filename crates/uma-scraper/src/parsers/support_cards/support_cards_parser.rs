use super::interpolation::{interpolate_effect, lb_levels_for_rarity};
use super::unique_effect::parse_unique_effect;
use crate::client::ScraperClient;
use crate::error::{ScraperError, ScraperResult};
use crate::url_resolver::{resolve_predicted_release_dates_url, resolve_support_cards_url};
use chrono::NaiveDate;
use log::{info, warn};
use serde_json::Value;
use std::collections::HashMap;
use uma_core::ids::SkillId;
use uma_core::support_card_skill::{HintAcquisition, SupportCardSkill};
use uma_core::{
    ids::SupportCardId,
    models::support_card::{CardType, EffectValue, LbValues, Rarity, SupportCard},
};

pub async fn fetch_support_card_roster(client: &ScraperClient) -> ScraperResult<Vec<SupportCard>> {
    let predicted_dates = fetch_support_card_predicted_release_dates(client).await?;
    let url = resolve_support_cards_url(client).await?;
    let json = client.fetch(&url).await?;
    parse_support_card_roster(&json, &predicted_dates)
}

pub async fn fetch_support_card_predicted_release_dates(
    client: &ScraperClient,
) -> ScraperResult<HashMap<SupportCardId, NaiveDate>> {
    let url = resolve_predicted_release_dates_url(client).await?;
    let json = client.fetch(&url).await?;
    parse_support_card_predicted_release_dates(&json)
}

fn parse_support_card_predicted_release_dates(
    json: &str,
) -> ScraperResult<HashMap<SupportCardId, NaiveDate>> {
    let root: Value = serde_json::from_str(json).map_err(|e| {
        ScraperError::JsonError(format!(
            "failed to parse support card predicted release dates JSON: {e}"
        ))
    })?;

    let support_cards = root["support_cards"]
        .as_object()
        .ok_or_else(|| ScraperError::MissingField("support_cards".into()))?;

    let mut map = HashMap::new();

    for (id_str, entry) in support_cards {
        let id = id_str.parse::<u32>().map(SupportCardId).map_err(|_| {
            ScraperError::UnknownValue(format!("non-numeric support_cards key: {id_str}"))
        })?;

        let date_str = entry["release_date"]
            .as_str()
            .ok_or_else(|| ScraperError::MissingField(format!("release_date for {id_str}")))?;

        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
            ScraperError::InvalidDate(format!("invalid release_date for {id_str}: {e}"))
        })?;

        map.insert(id, date);
    }

    Ok(map)
}

fn parse_support_card_roster(
    json: &str,
    predicted_dates: &HashMap<SupportCardId, NaiveDate>,
) -> ScraperResult<Vec<SupportCard>> {
    let items: Vec<Value> = serde_json::from_str(json)
        .map_err(|e| ScraperError::JsonError(format!("failed to parse support cards JSON: {e}")))?;

    let mut cards = Vec::new();
    let mut skip_reasons: HashMap<&str, usize> = HashMap::new();

    for item in &items {
        let id = item["support_id"].as_u64().unwrap_or(0);
        match parse_support_card(item, predicted_dates) {
            Ok(Some(card)) => cards.push(card),
            Ok(None) => {}
            Err(e) => {
                let reason = match &e {
                    ScraperError::MissingField(_) => "Missing field",
                    ScraperError::UnknownValue(_) => "Unknown value",
                    ScraperError::InvalidDate(_) => "Invalid date",
                    ScraperError::InvalidShape(_) => "Invalid shape",
                    ScraperError::JsonError(_) => "JSON deserialization error",
                    _ => "Other",
                };
                warn!("Failed to parse support card support_id {id}: {e}");
                *skip_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    let skipped = skip_reasons.values().sum::<usize>();
    info!(
        "Support card roster parsing complete: {} parsed, {} skipped out of {} total",
        cards.len(),
        skipped,
        items.len()
    );

    if !skip_reasons.is_empty() {
        info!("Parse failure breakdown:");
        let mut reasons: Vec<_> = skip_reasons.iter().collect();
        reasons.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (reason, count) in reasons {
            info!("  {count}x {reason}");
        }
    }

    Ok(cards)
}

fn parse_rarity(value: &Value) -> ScraperResult<Rarity> {
    match value.as_u64() {
        Some(1) => Ok(Rarity::R),
        Some(2) => Ok(Rarity::SR),
        Some(3) => Ok(Rarity::SSR),
        _ => Err(ScraperError::UnknownValue(format!(
            "support card rarity: {value}"
        ))),
    }
}

fn parse_card_type(value: &Value) -> ScraperResult<CardType> {
    match value.as_str() {
        Some("speed") => Ok(CardType::Speed),
        Some("stamina") => Ok(CardType::Stamina),
        Some("power") => Ok(CardType::Power),
        Some("guts") => Ok(CardType::Guts),
        Some("intelligence") => Ok(CardType::Wit),
        Some("friend") => Ok(CardType::Friend),
        Some("group") => Ok(CardType::Group),
        _ => Err(ScraperError::UnknownValue(format!(
            "support card type: {value}"
        ))),
    }
}

fn parse_release(
    item: &Value,
    predicted_dates: &HashMap<SupportCardId, NaiveDate>,
) -> ScraperResult<(Option<NaiveDate>, bool)> {
    match item["release_en"].as_str() {
        Some(date_str) => {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| ScraperError::InvalidDate(format!("invalid release_en: {e}")))?;
            Ok((Some(date), false))
        }
        None => {
            let support_id = item["support_id"].as_u64().map(|n| SupportCardId(n as u32));
            match support_id.and_then(|id| predicted_dates.get(&id)) {
                Some(&date) => Ok((Some(date), true)),
                None => Ok((None, false)),
            }
        }
    }
}

fn parse_effects(item: &Value, rarity: &Rarity) -> ScraperResult<Vec<EffectValue>> {
    let lb_levels = lb_levels_for_rarity(rarity);

    let effects_array = item["effects"]
        .as_array()
        .ok_or_else(|| ScraperError::MissingField("effects".into()))?;

    let mut effects = Vec::new();

    for row in effects_array {
        let cols = row
            .as_array()
            .ok_or_else(|| ScraperError::InvalidShape("effect row is not an array".into()))?;

        if cols.is_empty() {
            continue;
        }

        let effect_id = cols[0]
            .as_u64()
            .ok_or_else(|| ScraperError::MissingField("effect_id in row".into()))?
            as u32;

        let matrix_values: Vec<i64> = cols[1..].iter().map(|v| v.as_i64().unwrap_or(-1)).collect();

        let compute = |lb_index: usize| -> Option<i32> {
            let level = lb_levels[lb_index];
            let val = interpolate_effect(&matrix_values, level);
            if val <= 0 { None } else { Some(val as i32) }
        };

        let lb_values = LbValues {
            lb0: compute(0),
            lb1: compute(1),
            lb2: compute(2),
            lb3: compute(3),
            mlb: compute(4),
        };

        if let Some(effect) = EffectValue::from_id(effect_id, lb_values) {
            effects.push(effect);
        }
    }

    Ok(effects)
}

fn parse_skills(item: &Value) -> Vec<SupportCardSkill> {
    let mut skills = Vec::new();

    if let Some(arr) = item["event_skills"].as_array() {
        for v in arr {
            if let Some(n) = v.as_u64() {
                skills.push(SupportCardSkill {
                    id: SkillId(n as u32),
                    acquisition: HintAcquisition::Event,
                });
            }
        }
    }

    if let Some(hints) = item["hints"]["hint_skills"].as_array() {
        for v in hints {
            if let Some(n) = v.as_u64() {
                skills.push(SupportCardSkill {
                    id: SkillId(n as u32),
                    acquisition: HintAcquisition::Hint,
                });
            }
        }
    }

    skills
}

fn parse_support_card(
    item: &Value,
    predicted_dates: &HashMap<SupportCardId, NaiveDate>,
) -> ScraperResult<Option<SupportCard>> {
    let id = item["support_id"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField("support_id".into()))
        .map(|n| SupportCardId(n as u32))?;

    let char_id = item["char_id"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField("char_id".into()))? as u32;

    let char_name = item["char_name"]
        .as_str()
        .ok_or_else(|| ScraperError::MissingField("char_name".into()))?
        .to_string();

    let title = item["title_en"].as_str().unwrap_or("").to_string();

    let rarity = parse_rarity(&item["rarity"])?;
    let card_type = parse_card_type(&item["type"])?;
    let is_welfare = item["obtained"].as_str() == Some("welfare");
    let (release_date, is_predicted_date) = parse_release(item, predicted_dates)?;

    let unique_effect = match item.get("unique") {
        Some(unique) => Some(parse_unique_effect(unique)?),
        None => None,
    };

    let effects = parse_effects(item, &rarity)?;
    let skills = parse_skills(item);

    Ok(Some(SupportCard {
        id,
        char_id,
        char_name,
        title,
        card_type,
        rarity,
        is_welfare,
        release_date,
        is_predicted_date,
        unique_effect,
        effects,
        skills,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_predicted_dates() -> HashMap<SupportCardId, NaiveDate> {
        HashMap::new()
    }

    fn valid_item() -> Value {
        serde_json::json!({
            "support_id": 10001,
            "char_id": 1001,
            "char_name": "Special Week",
            "title_en": "[Tracen Academy]",
            "type": "guts",
            "rarity": 1,
            "obtained": "gacha",
            "release_en": "2025-06-26",
            "effects": [
                [1, 5, -1, -1, 10, 10, -1, -1, 15, -1, -1, -1],
                [2, 10, -1, -1, -1, 25, -1, -1, -1, 35, -1, -1]
            ],
            "event_skills": [200042],
            "hints": {
                "hint_skills": [200162, 200232]
            }
        })
    }

    #[test]
    fn parses_valid_card() {
        let card = parse_support_card(&valid_item(), &no_predicted_dates())
            .unwrap()
            .unwrap();

        assert_eq!(card.id, SupportCardId(10001));
        assert_eq!(card.char_name, "Special Week");
        assert_eq!(card.title, "[Tracen Academy]");
        assert!(!card.is_welfare);
        assert!(!card.is_predicted_date);
        assert!(card.unique_effect.is_none());
        assert_eq!(card.effects.len(), 2);
        assert_eq!(card.skills.len(), 3);
    }

    #[test]
    fn parses_event_skills() {
        let card = parse_support_card(&valid_item(), &no_predicted_dates())
            .unwrap()
            .unwrap();

        let event_skills: Vec<_> = card
            .skills
            .iter()
            .filter(|s| s.acquisition == HintAcquisition::Event)
            .collect();

        assert_eq!(event_skills.len(), 1);
        assert_eq!(event_skills[0].id, SkillId(200042));
    }

    #[test]
    fn parses_hint_skills() {
        let card = parse_support_card(&valid_item(), &no_predicted_dates())
            .unwrap()
            .unwrap();

        let hint_skills: Vec<_> = card
            .skills
            .iter()
            .filter(|s| s.acquisition == HintAcquisition::Hint)
            .collect();

        assert_eq!(hint_skills.len(), 2);
        assert_eq!(hint_skills[0].id, SkillId(200162));
        assert_eq!(hint_skills[1].id, SkillId(200232));
    }

    #[test]
    fn handles_missing_skills_gracefully() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("event_skills");
        item["hints"].as_object_mut().unwrap().remove("hint_skills");
        let card = parse_support_card(&item, &no_predicted_dates())
            .unwrap()
            .unwrap();
        assert_eq!(card.skills.len(), 0);
    }

    #[test]
    fn welfare_card_sets_is_welfare() {
        let mut item = valid_item();
        item["obtained"] = serde_json::json!("welfare");
        let card = parse_support_card(&item, &no_predicted_dates())
            .unwrap()
            .unwrap();
        assert!(card.is_welfare);
    }

    #[test]
    fn drops_unknown_effect_id_with_warning() {
        let mut item = valid_item();
        item["effects"] = serde_json::json!([
            [1, 5, -1, -1, 10, 10, -1, -1, 15, -1, -1, -1],
            [999, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]
        ]);
        let card = parse_support_card(&item, &no_predicted_dates())
            .unwrap()
            .unwrap();
        assert_eq!(card.effects.len(), 1);
    }

    #[test]
    fn uses_predicted_date_when_no_release_en() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("release_en");
        let date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
        let mut predicted = HashMap::new();
        predicted.insert(SupportCardId(10001), date);
        let card = parse_support_card(&item, &predicted).unwrap().unwrap();
        assert_eq!(card.release_date, Some(date));
        assert!(card.is_predicted_date);
    }

    #[test]
    fn errors_on_invalid_rarity() {
        let mut item = valid_item();
        item["rarity"] = serde_json::json!(99);
        let result = parse_support_card(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::UnknownValue(_))));
    }

    #[test]
    fn errors_on_invalid_card_type() {
        let mut item = valid_item();
        item["type"] = serde_json::json!("invalid");
        let result = parse_support_card(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::UnknownValue(_))));
    }
}
