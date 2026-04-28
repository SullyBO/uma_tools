use crate::client::ScraperClient;
use crate::error::{ScraperError, ScraperResult};
use chrono::{DateTime, Utc};
use log::info;
use serde_json::Value;
use uma_core::{
    ids::{SkillId, UmaId},
    models::uma::{
        AptitudeLevel, Aptitudes, BaseStats, DistanceAptitudes, GrowthRates, Rarity,
        StrategyAptitudes, SurfaceAptitudes, Uma,
    },
    uma_skill::{SkillAcquisition, UmaSkill},
};

const CHARACTER_CARDS_URL: &str =
    "https://gametora.com/data/umamusume/character-cards.94362f9a.json";

pub async fn fetch_uma_roster(client: &ScraperClient) -> ScraperResult<Vec<Uma>> {
    let json = client.fetch(CHARACTER_CARDS_URL).await?;
    parse_uma_roster(&json)
}

fn parse_uma_roster(json: &str) -> ScraperResult<Vec<Uma>> {
    let now = Utc::now();
    let items: Vec<Value> = serde_json::from_str(json).map_err(|e| {
        ScraperError::JsonError(format!("failed to parse character cards JSON: {e}"))
    })?;

    let mut umas = Vec::new();
    let mut skipped_jp = 0usize;
    let mut skip_reasons: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for item in &items {
        match parse_uma_item(item, now) {
            Ok(Some(uma)) => umas.push(uma),
            Ok(None) => skipped_jp += 1,
            Err(e) => {
                let id = item["card_id"].as_u64().unwrap_or(0);
                let reason = match &e {
                    ScraperError::MissingField(_) => "Missing field",
                    ScraperError::UnknownValue(_) => "Unknown value",
                    ScraperError::InvalidCondition(_) => "Invalid condition",
                    ScraperError::InvalidDate(_) => "Invalid date",
                    ScraperError::InvalidShape(_) => "Invalid shape",
                    ScraperError::JsonError(_) => "JSON deserialization error",
                    _ => "Other",
                };
                log::warn!("Failed to parse uma card_id {id}: {e}");
                *skip_reasons.entry(reason).or_insert(0) += 1;
            }
        }
    }

    let skipped_parse = skip_reasons.values().sum::<usize>();

    info!(
        "Character roster parsing complete: {} parsed, {} skipped (JP-only: {}) out of {} total",
        umas.len(),
        skipped_jp + skipped_parse,
        skipped_jp,
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

    Ok(umas)
}

fn parse_uma_item(item: &Value, now: DateTime<Utc>) -> ScraperResult<Option<Uma>> {
    let release_en = item["release_en"].as_str();

    match release_en {
        None => return Ok(None),
        Some(date_str) => {
            let release_date = DateTime::parse_from_str(
                &format!("{date_str}T00:00:00+00:00"),
                "%Y-%m-%dT%H:%M:%S%z",
            )
            .map_err(|e| ScraperError::InvalidDate(format!("invalid release_en date: {e}")))?
            .with_timezone(&Utc);

            if release_date > now {
                return Ok(None);
            }
        }
    }

    let id = item["card_id"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField("card_id".into()))
        .map(|n| UmaId(n as u32))?;

    let name = item["name_en"]
        .as_str()
        .ok_or_else(|| ScraperError::MissingField("name_en".into()))?
        .to_string();

    let subtitle = item["title_en_gl"]
        .as_str()
        .ok_or_else(|| ScraperError::MissingField("title_en_gl".into()))?
        .to_string();

    let rarity = parse_rarity(&item["rarity"])?;
    let base_stats = parse_base_stats(&item["base_stats"])?;
    let growth_rates = parse_growth_rates(&item["stat_bonus"])?;
    let aptitudes = parse_aptitudes(&item["aptitude"])?;
    let skill_list = parse_skills(item)?;

    let uma = Uma {
        id,
        name,
        subtitle,
        rarity,
        base_stats,
        growth_rates,
        aptitudes,
        skill_list,
    };

    Ok(Some(uma))
}

fn parse_rarity(value: &Value) -> ScraperResult<Rarity> {
    match value.as_u64() {
        Some(1) => Ok(Rarity::R),
        Some(2) => Ok(Rarity::SR),
        Some(3) => Ok(Rarity::SSR),
        _ => Err(ScraperError::UnknownValue(format!("rarity value: {value}"))),
    }
}

fn parse_base_stats(value: &Value) -> ScraperResult<BaseStats> {
    let arr = value
        .as_array()
        .ok_or_else(|| ScraperError::MissingField("base_stats is not an array".into()))?;

    if arr.len() < 5 {
        return Err(ScraperError::InvalidShape(
            "base_stats has fewer than 5 elements".into(),
        ));
    }

    let mut stats = arr.iter().map(|v| {
        v.as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| ScraperError::UnknownValue("base_stats element is not a number".into()))
    });

    Ok(BaseStats {
        speed: stats.next().unwrap()?,
        stamina: stats.next().unwrap()?,
        power: stats.next().unwrap()?,
        guts: stats.next().unwrap()?,
        wit: stats.next().unwrap()?,
    })
}

fn parse_growth_rates(value: &Value) -> ScraperResult<GrowthRates> {
    let arr = value
        .as_array()
        .ok_or_else(|| ScraperError::MissingField("stat_bonus is not an array".into()))?;

    if arr.len() < 5 {
        return Err(ScraperError::InvalidShape(
            "stat_bonus has fewer than 5 elements".into(),
        ));
    }

    let mut stats = arr.iter().map(|v| {
        v.as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| ScraperError::UnknownValue("stat_bonus element is not a number".into()))
    });

    Ok(GrowthRates {
        speed: stats.next().unwrap()?,
        stamina: stats.next().unwrap()?,
        power: stats.next().unwrap()?,
        guts: stats.next().unwrap()?,
        wit: stats.next().unwrap()?,
    })
}

fn parse_aptitude_level(value: &Value) -> ScraperResult<AptitudeLevel> {
    match value.as_str() {
        Some("A") => Ok(AptitudeLevel::A),
        Some("B") => Ok(AptitudeLevel::B),
        Some("C") => Ok(AptitudeLevel::C),
        Some("D") => Ok(AptitudeLevel::D),
        Some("E") => Ok(AptitudeLevel::E),
        Some("F") => Ok(AptitudeLevel::F),
        Some("G") => Ok(AptitudeLevel::G),
        _ => Err(ScraperError::UnknownValue(format!(
            "aptitude level: {value}"
        ))),
    }
}

fn parse_aptitudes(value: &Value) -> ScraperResult<Aptitudes> {
    let arr = value
        .as_array()
        .ok_or_else(|| ScraperError::MissingField("aptitude is not an array".into()))?;

    if arr.len() < 10 {
        return Err(ScraperError::InvalidShape(
            "aptitude has fewer than 10 elements".into(),
        ));
    }

    Ok(Aptitudes {
        surface: SurfaceAptitudes {
            turf: parse_aptitude_level(&arr[0])?,
            dirt: parse_aptitude_level(&arr[1])?,
        },
        distance: DistanceAptitudes {
            short: parse_aptitude_level(&arr[2])?,
            mile: parse_aptitude_level(&arr[3])?,
            medium: parse_aptitude_level(&arr[4])?,
            long: parse_aptitude_level(&arr[5])?,
        },
        strategy: StrategyAptitudes {
            front: parse_aptitude_level(&arr[6])?,
            pace: parse_aptitude_level(&arr[7])?,
            late: parse_aptitude_level(&arr[8])?,
            end: parse_aptitude_level(&arr[9])?,
        },
    })
}

fn parse_skills(item: &Value) -> ScraperResult<Vec<UmaSkill>> {
    let mut skills = Vec::new();

    let flat_categories = [
        ("skills_unique", SkillAcquisition::Unique),
        ("skills_innate", SkillAcquisition::Innate),
        ("skills_awakening", SkillAcquisition::Awakening),
        ("skills_event", SkillAcquisition::Event),
    ];

    for (key, acquisition) in flat_categories {
        if let Some(arr) = item[key].as_array() {
            for v in arr {
                if let Some(n) = v.as_u64() {
                    skills.push(UmaSkill {
                        id: SkillId(n as u32),
                        acquisition,
                    });
                }
            }
        }
    }

    if let Some(arr) = item["skills_evo"].as_array() {
        for v in arr {
            let new_id = v["new"]
                .as_u64()
                .ok_or_else(|| ScraperError::MissingField("'new' in skills_evo entry".into()))
                .map(|n| SkillId(n as u32))?;

            let old_id = v["old"]
                .as_u64()
                .ok_or_else(|| ScraperError::MissingField("'old' in skills_evo entry".into()))
                .map(|n| SkillId(n as u32))?;

            skills.push(UmaSkill {
                id: new_id,
                acquisition: SkillAcquisition::Evolution(old_id),
            });
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_item() -> serde_json::Value {
        serde_json::json!({
            "card_id": 102001,
            "name_en": "Seiun Sky",
            "title_en_gl": "[Reeling in the Big One]",
            "rarity": 3,
            "base_stats": [98, 98, 88, 83, 83],
            "stat_bonus": [20, 0, 10, 0, 20],
            "aptitude": ["A", "G", "G", "C", "A", "A", "A", "B", "D", "E"],
            "skills_unique": [100201],
            "skills_innate": [200881, 201192],
            "skills_awakening": [201522],
            "skills_event": [200891, 200742],
            "skills_evo": [
                {"new": 102001111, "old": 201191},
                {"new": 102001211, "old": 200541}
            ],
            "release_en": "2025-06-26"
        })
    }

    #[test]
    fn parses_valid_item() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T00:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let uma = parse_uma_item(&valid_item(), now).unwrap().unwrap();

        assert_eq!(uma.id, UmaId(102001));
        assert_eq!(uma.name, "Seiun Sky");
        assert_eq!(uma.subtitle, "[Reeling in the Big One]");
        assert!(matches!(uma.rarity, Rarity::SSR));
        assert_eq!(uma.base_stats.speed, 98);
        assert_eq!(uma.base_stats.stamina, 98);
        assert_eq!(uma.base_stats.power, 88);
        assert_eq!(uma.base_stats.guts, 83);
        assert_eq!(uma.base_stats.wit, 83);
        assert_eq!(uma.growth_rates.speed, 20);
        assert_eq!(uma.growth_rates.stamina, 0);
        assert_eq!(uma.growth_rates.power, 10);
        assert_eq!(uma.growth_rates.guts, 0);
        assert_eq!(uma.growth_rates.wit, 20);
        assert!(matches!(uma.aptitudes.surface.turf, AptitudeLevel::A));
        assert!(matches!(uma.aptitudes.surface.dirt, AptitudeLevel::G));
        assert!(matches!(uma.aptitudes.distance.short, AptitudeLevel::G));
        assert!(matches!(uma.aptitudes.distance.mile, AptitudeLevel::C));
        assert!(matches!(uma.aptitudes.strategy.front, AptitudeLevel::A));
        assert!(matches!(uma.aptitudes.strategy.end, AptitudeLevel::E));
        assert_eq!(uma.skill_list.len(), 8); // 1 unique + 2 innate + 1 awakening + 2 event + 2 evo
    }

    #[test]
    fn skips_jp_only_no_release_en() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("release_en");
        let now = Utc::now();
        let result = parse_uma_item(&item, now).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn skips_unreleased() {
        let mut item = valid_item();
        item["release_en"] = serde_json::json!("2099-01-01");
        let now = Utc::now();
        let result = parse_uma_item(&item, now).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn errors_on_invalid_rarity() {
        let mut item = valid_item();
        item["rarity"] = serde_json::json!(99);
        let now = Utc::now();
        let result = parse_uma_item(&item, now);
        assert!(matches!(result, Err(ScraperError::UnknownValue(_))));
    }

    #[test]
    fn errors_on_short_base_stats() {
        let mut item = valid_item();
        item["base_stats"] = serde_json::json!([98, 98, 88]);
        let now = Utc::now();
        let result = parse_uma_item(&item, now);
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn errors_on_short_aptitude() {
        let mut item = valid_item();
        item["aptitude"] = serde_json::json!(["A", "B", "C"]);
        let now = Utc::now();
        let result = parse_uma_item(&item, now);
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn handles_missing_skill_categories() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("skills_event");
        item.as_object_mut().unwrap().remove("skills_awakening");
        let now = Utc::now();
        let uma = parse_uma_item(&item, now).unwrap().unwrap();
        assert_eq!(uma.skill_list.len(), 5); // 1 unique + 2 innate + 2 evo
    }

    #[test]
    fn errors_on_short_stat_bonus() {
        let mut item = valid_item();
        item["stat_bonus"] = serde_json::json!([20, 0, 10]);
        let now = Utc::now();
        let result = parse_uma_item(&item, now);
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn parses_evolution_skills() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T00:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let uma = parse_uma_item(&valid_item(), now).unwrap().unwrap();

        let evo_skills: Vec<_> = uma
            .skill_list
            .iter()
            .filter(|s| matches!(s.acquisition, SkillAcquisition::Evolution(_)))
            .collect();

        assert_eq!(evo_skills.len(), 2);
        assert_eq!(evo_skills[0].id, SkillId(102001111));
        assert!(matches!(
            evo_skills[0].acquisition,
            SkillAcquisition::Evolution(SkillId(201191))
        ));
    }
}
