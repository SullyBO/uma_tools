use super::effect_parser::parse_effect;
use super::icon_category::icon_id_to_category;
use crate::client::ScraperClient;
use crate::error::{ScraperError, ScraperResult};
use crate::url_resolver::resolve_skills_url;
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use uma_core::{
    ids::SkillId,
    models::skill::{Rarity, Skill},
};

pub async fn fetch_skill_roster(client: &ScraperClient) -> ScraperResult<Vec<Skill>> {
    let url = resolve_skills_url(client).await?;
    let json = client.fetch(&url).await?;
    parse_skill_roster(&json)
}

fn parse_skill_roster(json: &str) -> ScraperResult<Vec<Skill>> {
    let items: Vec<Value> = serde_json::from_str(json)
        .map_err(|e| ScraperError::JsonError(format!("failed to parse skills JSON: {e}")))?;

    let mut skills = Vec::new();
    let mut skip_reasons: HashMap<&str, usize> = HashMap::new();
    let mut jp_only_count = 0usize;

    for item in &items {
        match parse_skill_item(item) {
            Ok(skill) => {
                if skill.is_jp_only {
                    jp_only_count += 1;
                }
                skills.push(skill);
            }
            Err(e) => {
                let id = item["id"].as_u64().unwrap_or(0);
                let reason = match &e {
                    ScraperError::MissingField(_) => "Missing field",
                    ScraperError::UnknownValue(_) => "Unknown value",
                    ScraperError::InvalidCondition(_) => "Invalid condition",
                    ScraperError::InvalidDate(_) => "Invalid date",
                    ScraperError::JsonError(_) => "JSON deserialization error",
                    _ => "Other",
                };
                log::warn!("Failed to parse skill id {id}: {e}");
                *skip_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    info!("Skill roster parsing complete:");
    info!("{} global skills parsed", skills.len() - jp_only_count);
    info!("{} JP-only skills parsed", jp_only_count);
    info!(
        "{} skipped skills out of {} total",
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

    Ok(skills)
}

fn parse_skill_item(item: &Value) -> ScraperResult<Skill> {
    let id = item["id"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField("id".into()))
        .map(|n| SkillId(n as u32))?;

    let rarity = parse_rarity(&item["rarity"], id)?;

    let (name, is_jp_only) = match item["name_en"].as_str() {
        Some(n) => (n.to_string(), false),
        None => {
            let name = item["enname"]
                .as_str()
                .ok_or_else(|| ScraperError::MissingField(format!("enname for skill {}", id.0)))?
                .to_string();
            (name, true)
        }
    };

    let ingame_description = item["desc_en"].as_str().unwrap_or("").to_string();

    let condition_groups = item
        .pointer("/loc/en/condition_groups")
        .unwrap_or(&item["condition_groups"]);

    let icon_id = item["iconid"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField(format!("iconid for skill {}", id.0)))?
        as u32;

    let category = icon_id_to_category(icon_id).ok_or_else(|| {
        ScraperError::UnknownValue(format!("iconid {icon_id} for skill {}", id.0))
    })?;

    let sp_cost = item["cost"].as_u64().unwrap_or(0) as u32;

    let effects = condition_groups
        .as_array()
        .map(|groups| groups.iter().map(|cg| parse_effect(cg, id)).collect())
        .unwrap_or(Ok(Vec::new()))?;

    Ok(Skill {
        id,
        name,
        ingame_description,
        category,
        rarity,
        sp_cost,
        effects,
        is_jp_only,
    })
}

fn parse_rarity(value: &Value, id: SkillId) -> ScraperResult<Rarity> {
    match value.as_u64() {
        Some(1) => Ok(Rarity::Normal),
        Some(2) => Ok(Rarity::Rare),
        Some(3) => Ok(Rarity::Rare),
        Some(4) => Ok(Rarity::Unique),
        Some(5) => Ok(Rarity::Unique),
        Some(6) => Ok(Rarity::Evolution),
        _ => Err(ScraperError::UnknownValue(format!(
            "rarity value {} for skill {}",
            value, id.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uma_core::models::skill::Duration;

    fn valid_item() -> serde_json::Value {
        serde_json::json!({
            "id": 10071,
            "name_en": "Warning Shot!",
            "desc_en": "Slightly increase velocity with a long spurt starting halfway through the race.",
            "rarity": 3,
            "iconid": 20013,
            "cost": 200,
            "condition_groups": [
                {
                    "base_time": 50000,
                    "effects": [
                        {"type": 27, "value": 4500},
                        {"type": 31, "value": 2000}
                    ],
                    "condition": "distance_rate>=50&order_rate>50",
                    "precondition": "phase>=2"
                }
            ]
        })
    }

    #[test]
    fn parses_valid_skill() {
        let skill = parse_skill_item(&valid_item()).unwrap();
        assert_eq!(skill.id, SkillId(10071));
        assert_eq!(skill.name, "Warning Shot!");
        assert_eq!(
            skill.ingame_description,
            "Slightly increase velocity with a long spurt starting halfway through the race."
        );
        assert!(matches!(skill.rarity, Rarity::Rare));
        assert_eq!(skill.sp_cost, 200);
    }

    #[test]
    fn parses_full_roster() {
        let json = serde_json::json!([valid_item(), valid_item()]).to_string();
        let skills = parse_skill_roster(&json).unwrap();
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn roster_tolerates_bad_items() {
        let bad_item = serde_json::json!({"id": 99999, "rarity": 99});
        let json = serde_json::json!([valid_item(), bad_item]).to_string();
        let skills = parse_skill_roster(&json).unwrap();
        assert_eq!(skills.len(), 1);
    }

    #[test]
    fn prefers_en_loc_condition_groups() {
        let mut item = valid_item();
        item["loc"] = serde_json::json!({
            "en": {
                "condition_groups": [{
                    "base_time": 50000,
                    "effects": [{"type": 27, "value": 1500}],
                    "condition": "is_lastspurt==1",
                    "precondition": ""
                }]
            }
        });
        let skill = parse_skill_item(&item).unwrap();
        let conditions = &skill.effects[0].conditions;
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].cond_key, "is_lastspurt");
    }

    #[test]
    fn falls_back_to_top_level_condition_groups_when_no_en_loc() {
        let item = valid_item();
        let skill = parse_skill_item(&item).unwrap();
        assert_eq!(skill.effects[0].conditions[0].cond_key, "distance_rate");
    }

    #[test]
    fn errors_on_missing_id() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("id");
        assert!(matches!(
            parse_skill_item(&item),
            Err(ScraperError::MissingField(_))
        ));
    }

    #[test]
    fn errors_on_missing_name() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("name_en");
        assert!(matches!(
            parse_skill_item(&item),
            Err(ScraperError::MissingField(_))
        ));
    }

    #[test]
    fn errors_on_invalid_rarity() {
        let mut item = valid_item();
        item["rarity"] = serde_json::json!(99);
        assert!(matches!(
            parse_skill_item(&item),
            Err(ScraperError::UnknownValue(_))
        ));
    }

    #[test]
    fn errors_on_unknown_iconid() {
        let mut item = valid_item();
        item["iconid"] = serde_json::json!(99999);
        assert!(matches!(
            parse_skill_item(&item),
            Err(ScraperError::UnknownValue(_))
        ));
    }

    #[test]
    fn defaults_sp_cost_to_zero_when_absent() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("cost");
        let skill = parse_skill_item(&item).unwrap();
        assert_eq!(skill.sp_cost, 0);
    }

    #[test]
    fn defaults_desc_en_to_empty_when_absent() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("desc_en");
        let skill = parse_skill_item(&item).unwrap();
        assert_eq!(skill.ingame_description, "");
    }

    #[test]
    fn handles_empty_condition_groups() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("condition_groups");
        let skill = parse_skill_item(&item).unwrap();
        assert!(skill.effects.is_empty());
    }

    #[test]
    fn parses_timed_duration() {
        let skill = parse_skill_item(&valid_item()).unwrap();
        assert!(
            matches!(skill.effects[0].duration, Duration::Timed(v) if (v - 5.0).abs() < f32::EPSILON)
        );
    }
}
