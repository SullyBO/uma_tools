use crate::client::ScraperClient;
use crate::error::{ScraperError, ScraperResult};
use crate::url_resolver::{resolve_predicted_release_dates_url, resolve_uma_url};
use chrono::NaiveDate;
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use uma_core::{
    ids::{SkillId, UmaId},
    models::uma::{
        AptitudeLevel, Aptitudes, BaseStats, DistanceAptitudes, GrowthRates, Rarity,
        StrategyAptitudes, SurfaceAptitudes, Uma,
    },
    uma_skill::{SkillAcquisition, UmaSkill},
};

pub async fn fetch_uma_roster(client: &ScraperClient) -> ScraperResult<Vec<Uma>> {
    let predicted_dates = fetch_predicted_release_dates(client).await?;
    let url = resolve_uma_url(client).await?;
    let json = client.fetch(&url).await?;
    parse_uma_roster(&json, &predicted_dates)
}

pub async fn fetch_predicted_release_dates(
    client: &ScraperClient,
) -> ScraperResult<HashMap<UmaId, NaiveDate>> {
    let url = resolve_predicted_release_dates_url(client).await?;
    let json = client.fetch(&url).await?;
    parse_predicted_release_dates(&json)
}

fn parse_predicted_release_dates(json: &str) -> ScraperResult<HashMap<UmaId, NaiveDate>> {
    let root: Value = serde_json::from_str(json).map_err(|e| {
        ScraperError::JsonError(format!("failed to parse predicted release dates JSON: {e}"))
    })?;

    let char_cards = root["char_cards"]
        .as_object()
        .ok_or_else(|| ScraperError::MissingField("char_cards".into()))?;

    let mut map = HashMap::new();

    for (id_str, entry) in char_cards {
        let id = id_str.parse::<u32>().map(UmaId).map_err(|_| {
            ScraperError::UnknownValue(format!("non-numeric char_cards key: {id_str}"))
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

fn parse_uma_roster(
    json: &str,
    predicted_dates: &HashMap<UmaId, NaiveDate>,
) -> ScraperResult<Vec<Uma>> {
    let items: Vec<Value> = serde_json::from_str(json).map_err(|e| {
        ScraperError::JsonError(format!("failed to parse character cards JSON: {e}"))
    })?;

    let mut umas = Vec::new();
    let mut skip_reasons: HashMap<&str, usize> = HashMap::new();

    for item in &items {
        match parse_uma(item, predicted_dates) {
            Ok(Some(uma)) => umas.push(uma),
            Ok(None) => {}
            Err(e) => {
                let id = item["card_id"].as_u64().unwrap_or(0);
                let reason = match &e {
                    ScraperError::MissingField(_) => "Missing field",
                    ScraperError::UnknownValue(_) => "Unknown value",
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
        "Character roster parsing complete: {} parsed, {} skipped out of {} total",
        umas.len(),
        skipped_parse,
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

fn parse_uma(
    item: &Value,
    predicted_dates: &HashMap<UmaId, NaiveDate>,
) -> ScraperResult<Option<Uma>> {
    let id = item["card_id"]
        .as_u64()
        .ok_or_else(|| ScraperError::MissingField("card_id".into()))
        .map(|n| UmaId(n as u32))?;

    let Some((release_date, is_predicted_date)) = parse_release(item, id, predicted_dates)? else {
        return Ok(None);
    };

    let name = item["name_en"]
        .as_str()
        .ok_or_else(|| ScraperError::MissingField("name_en".into()))?
        .to_string();

    let subtitle = item["version"].as_str().unwrap_or("default").to_string();

    let rarity = parse_rarity(&item["rarity"])?;
    let base_stats = parse_base_stats(&item["base_stats"])?;
    let growth_rates = parse_growth_rates(&item["stat_bonus"])?;
    let aptitudes = parse_aptitudes(&item["aptitude"])?;
    let skill_list = parse_uma_skills(item)?;

    Ok(Some(Uma {
        id,
        name,
        subtitle,
        rarity,
        base_stats,
        growth_rates,
        aptitudes,
        skill_list,
        release_date,
        is_predicted_date,
    }))
}

fn parse_release(
    item: &Value,
    id: UmaId,
    predicted_dates: &HashMap<UmaId, NaiveDate>,
) -> ScraperResult<Option<(NaiveDate, bool)>> {
    match item["release_en"].as_str() {
        Some(date_str) => {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| ScraperError::InvalidDate(format!("invalid release_en date: {e}")))?;
            Ok(Some((date, false)))
        }
        None => match predicted_dates.get(&id) {
            Some(&date) => Ok(Some((date, true))),
            None => Ok(None),
        },
    }
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

fn parse_uma_skills(item: &Value) -> ScraperResult<Vec<UmaSkill>> {
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

    fn no_predicted_dates() -> HashMap<UmaId, NaiveDate> {
        HashMap::new()
    }

    fn valid_item() -> serde_json::Value {
        serde_json::json!({
            "card_id": 102001,
            "name_en": "Seiun Sky",
            "version": "default",
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
        let uma = parse_uma(&valid_item(), &no_predicted_dates())
            .unwrap()
            .unwrap();

        assert_eq!(uma.id, UmaId(102001));
        assert_eq!(uma.name, "Seiun Sky");
        assert_eq!(uma.subtitle, "default");
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
        assert_eq!(uma.skill_list.len(), 8);
        assert!(!uma.is_predicted_date);
    }

    #[test]
    fn skips_when_no_release_en_and_no_predicted_date() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("release_en");
        let result = parse_uma(&item, &no_predicted_dates()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn uses_predicted_date_when_no_release_en() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("release_en");
        let date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
        let mut predicted = HashMap::new();
        predicted.insert(UmaId(102001), date);
        let uma = parse_uma(&item, &predicted).unwrap().unwrap();
        assert_eq!(uma.release_date, date);
        assert!(uma.is_predicted_date);
    }

    #[test]
    fn stores_future_release_en_date() {
        let mut item = valid_item();
        item["release_en"] = serde_json::json!("2099-01-01");
        let uma = parse_uma(&item, &no_predicted_dates()).unwrap().unwrap();
        assert_eq!(
            uma.release_date,
            NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()
        );
        assert!(!uma.is_predicted_date);
    }

    #[test]
    fn errors_on_invalid_rarity() {
        let mut item = valid_item();
        item["rarity"] = serde_json::json!(99);
        let result = parse_uma(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::UnknownValue(_))));
    }

    #[test]
    fn errors_on_short_base_stats() {
        let mut item = valid_item();
        item["base_stats"] = serde_json::json!([98, 98, 88]);
        let result = parse_uma(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn errors_on_short_aptitude() {
        let mut item = valid_item();
        item["aptitude"] = serde_json::json!(["A", "B", "C"]);
        let result = parse_uma(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn handles_missing_skill_categories() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("skills_event");
        item.as_object_mut().unwrap().remove("skills_awakening");
        let uma = parse_uma(&item, &no_predicted_dates()).unwrap().unwrap();
        assert_eq!(uma.skill_list.len(), 5);
    }

    #[test]
    fn errors_on_short_stat_bonus() {
        let mut item = valid_item();
        item["stat_bonus"] = serde_json::json!([20, 0, 10]);
        let result = parse_uma(&item, &no_predicted_dates());
        assert!(matches!(result, Err(ScraperError::InvalidShape(_))));
    }

    #[test]
    fn parses_evolution_skills() {
        let uma = parse_uma(&valid_item(), &no_predicted_dates())
            .unwrap()
            .unwrap();

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
