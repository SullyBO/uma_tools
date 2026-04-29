use crate::client::ScraperClient;
use crate::error::{ScraperError, ScraperResult};
use crate::icon_category::icon_id_to_category;
use log::info;
use serde_json::Value;
use uma_core::{
    ids::SkillId,
    models::skill::{Condition, Effect, EffectType, Operator, Rarity, Skill},
};

const SKILLS_URL: &str = "https://gametora.com/data/umamusume/skills.3b4d4239.json";

pub async fn fetch_skill_roster(client: &ScraperClient) -> ScraperResult<Vec<Skill>> {
    let json = client.fetch(SKILLS_URL).await?;
    parse_skill_roster(&json)
}

fn parse_skill_roster(json: &str) -> ScraperResult<Vec<Skill>> {
    let items: Vec<Value> = serde_json::from_str(json)
        .map_err(|e| ScraperError::JsonError(format!("failed to parse skills JSON: {e}")))?;

    let mut skills = Vec::new();
    let mut skip_reasons: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
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

    let icon_id = item["iconid"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField(format!("iconid for skill {}", id.0)))?
        as u32;

    let category = icon_id_to_category(icon_id).ok_or_else(|| {
        ScraperError::UnknownValue(format!("iconid {icon_id} for skill {}", id.0))
    })?;

    let sp_cost = item["cost"].as_u64().unwrap_or(0) as u32;
    let effects = parse_effects(item, id)?;

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

fn parse_effects(item: &Value, skill_id: SkillId) -> ScraperResult<Vec<Effect>> {
    let Some(groups) = item["condition_groups"].as_array() else {
        return Ok(Vec::new());
    };

    groups
        .iter()
        .map(|cg| parse_effect_group(cg, skill_id))
        .collect()
}

fn parse_effect_group(cg: &Value, skill_id: SkillId) -> ScraperResult<Effect> {
    let effects = cg["effects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let type_id = e["type"].as_u64()?;
            let value = e["value"].as_i64().unwrap_or(0);
            EffectType::from_raw(type_id, value)
        })
        .collect();

    let conditions = parse_condition_string(cg["condition"].as_str().unwrap_or(""), skill_id)?;
    let preconditions =
        parse_condition_string(cg["precondition"].as_str().unwrap_or(""), skill_id)?;

    Ok(Effect {
        effects,
        conditions,
        preconditions,
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

fn parse_condition_string(s: &str, skill_id: SkillId) -> ScraperResult<Vec<Condition>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    let mut conditions = Vec::new();

    for (or_idx, or_group) in s.split('@').enumerate() {
        for (and_idx, part) in or_group.split('&').enumerate() {
            let (cond_key, operator, cond_val) = parse_condition_part(part).ok_or_else(|| {
                ScraperError::InvalidCondition(format!("'{part}' in skill {}", skill_id.0))
            })?;

            let is_or = or_idx > 0 && and_idx == 0;

            conditions.push(Condition {
                cond_key,
                operator,
                cond_val,
                is_or,
            });
        }
    }

    Ok(conditions)
}

fn parse_condition_part(s: &str) -> Option<(String, Operator, String)> {
    let operators = [
        (">=", Operator::GtEq),
        ("<=", Operator::LtEq),
        ("!=", Operator::NotEq),
        ("==", Operator::Eq),
        (">", Operator::Gt),
        ("<", Operator::Lt),
    ];

    for (sym, op) in operators {
        if let Some(pos) = s.find(sym) {
            let key = s[..pos].trim().to_string();
            let val = s[pos + sym.len()..].trim().to_string();
            if !key.is_empty() && !val.is_empty() {
                return Some((key, op, val));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parses_effects() {
        let skill = parse_skill_item(&valid_item()).unwrap();
        assert_eq!(skill.effects.len(), 1);
        let effect = &skill.effects[0];
        assert_eq!(effect.effects.len(), 2);
        assert!(matches!(effect.effects[0], EffectType::TargetSpeedUp(_)));
        assert!(matches!(effect.effects[1], EffectType::AccelerationUp(_)));
    }

    #[test]
    fn parses_conditions() {
        let skill = parse_skill_item(&valid_item()).unwrap();
        let conditions = &skill.effects[0].conditions;
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].cond_key, "distance_rate");
        assert!(matches!(conditions[0].operator, Operator::GtEq));
        assert_eq!(conditions[0].cond_val, "50");
        assert!(!conditions[0].is_or);
        assert_eq!(conditions[1].cond_key, "order_rate");
        assert!(matches!(conditions[1].operator, Operator::Gt));
        assert_eq!(conditions[1].cond_val, "50");
        assert!(!conditions[1].is_or);
    }

    #[test]
    fn parses_preconditions() {
        let skill = parse_skill_item(&valid_item()).unwrap();
        let preconditions = &skill.effects[0].preconditions;
        assert_eq!(preconditions.len(), 1);
        assert_eq!(preconditions[0].cond_key, "phase");
        assert!(matches!(preconditions[0].operator, Operator::GtEq));
        assert_eq!(preconditions[0].cond_val, "2");
    }

    #[test]
    fn parses_or_conditions() {
        let mut item = valid_item();
        item["condition_groups"][0]["condition"] =
            serde_json::json!("distance_rate>=50&order<=3@distance_rate>=50&order_rate<=50");
        let skill = parse_skill_item(&item).unwrap();
        let conditions = &skill.effects[0].conditions;
        assert_eq!(conditions.len(), 4);
        assert!(!conditions[0].is_or);
        assert!(!conditions[1].is_or);
        assert!(conditions[2].is_or);
        assert!(!conditions[3].is_or);
    }

    #[test]
    fn skips_unknown_effect_types() {
        let mut item = valid_item();
        item["condition_groups"][0]["effects"] = serde_json::json!([
            {"type": 27, "value": 4500},
            {"type": 9999, "value": 100}
        ]);
        let skill = parse_skill_item(&item).unwrap();
        assert_eq!(skill.effects[0].effects.len(), 1);
    }

    #[test]
    fn handles_empty_condition_groups() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("condition_groups");
        let skill = parse_skill_item(&item).unwrap();
        assert!(skill.effects.is_empty());
    }

    #[test]
    fn handles_empty_conditions() {
        let mut item = valid_item();
        item["condition_groups"][0]["condition"] = serde_json::json!("");
        item["condition_groups"][0]["precondition"] = serde_json::json!("");
        let skill = parse_skill_item(&item).unwrap();
        assert!(skill.effects[0].conditions.is_empty());
        assert!(skill.effects[0].preconditions.is_empty());
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
    fn errors_on_malformed_condition() {
        let mut item = valid_item();
        item["condition_groups"][0]["condition"] = serde_json::json!("notacondition");
        assert!(matches!(
            parse_skill_item(&item),
            Err(ScraperError::InvalidCondition(_))
        ));
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
}
