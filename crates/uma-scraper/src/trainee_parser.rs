use crate::{
    client::ScraperClient,
    error::{ScraperError, ScraperResult},
    trainee_list_parser::TraineeIndexEntry,
};
use log::{info, warn};
use scraper::{Html, Selector};
use serde_json::Value;
use uma_core::{
    ids::{SkillId, UmaId},
    models::uma::{
        AptitudeLevel, Aptitudes, BaseStats, DistanceAptitudes, StrategyAptitudes,
        SurfaceAptitudes, Uma, UmaRarity,
    },
    uma_skill::{SkillAcquisition, UmaSkill},
};

pub async fn parse_all_characters(
    client: &ScraperClient,
    entries: &[TraineeIndexEntry],
) -> Vec<ScraperResult<Uma>> {
    let urls: Vec<String> = entries.iter().map(|e| e.gametora_url()).collect();
    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    let results = client.fetch_all(&url_refs).await;
    let mut parsed = Vec::new();
    let mut success = 0;
    let mut failed = 0;

    for (entry, result) in entries.iter().zip(results) {
        match result {
            Ok(html) => match parse_character_page(&html) {
                Ok(uma) => {
                    success += 1;
                    parsed.push(Ok(uma));
                }
                Err(e) => {
                    warn!(
                        "Failed to parse character {} ({}): {e}",
                        entry.name, entry.id.0
                    );
                    failed += 1;
                    parsed.push(Err(e));
                }
            },
            Err(e) => {
                warn!(
                    "Failed to fetch character {} ({}): {e}",
                    entry.name, entry.id.0
                );
                failed += 1;
                parsed.push(Err(e));
            }
        }
    }

    info!(
        "Character parsing complete: {} succeeded, {} failed out of {} total",
        success,
        failed,
        entries.len(),
    );

    parsed
}

pub fn parse_character_page(html: &str) -> ScraperResult<Uma> {
    let document = Html::parse_document(html);
    let script_sel = Selector::parse("script#__NEXT_DATA__").unwrap();

    let json_text = document
        .select(&script_sel)
        .next()
        .ok_or_else(|| ScraperError::ParseError("__NEXT_DATA__ script not found".into()))?
        .text()
        .collect::<String>();

    let root: Value = serde_json::from_str(&json_text)
        .map_err(|e| ScraperError::ParseError(format!("failed to parse __NEXT_DATA__: {e}")))?;

    let item = &root["props"]["pageProps"]["itemData"];

    let id = item["card_id"]
        .as_u64()
        .ok_or_else(|| ScraperError::ParseError("missing card_id".into()))
        .map(|n| UmaId(n as u32))?;

    let name = item["name_en"]
        .as_str()
        .ok_or_else(|| ScraperError::ParseError("missing name_en".into()))?
        .to_string();

    let subtitle = item["title_en_gl"]
        .as_str()
        .ok_or_else(|| ScraperError::ParseError("missing title_en_gl".into()))?
        .to_string();

    let rarity = parse_rarity(&item["rarity"])?;
    let base_stats = parse_base_stats(&item["base_stats"])?;
    let aptitudes = parse_aptitudes(&item["aptitude"])?;
    let skill_list = parse_skills(item)?;

    let uma = Uma {
        id,
        name,
        subtitle,
        rarity,
        base_stats,
        aptitudes,
        skill_list,
    };

    info!(
        "Parsed uma: {} {} (id: {}, skills: {})",
        uma.rarity,
        uma.name,
        uma.id.0,
        uma.skill_list.len(),
    );

    Ok(uma)
}

fn parse_rarity(value: &Value) -> ScraperResult<UmaRarity> {
    match value.as_u64() {
        Some(1) => Ok(UmaRarity::R),
        Some(2) => Ok(UmaRarity::SR),
        Some(3) => Ok(UmaRarity::SSR),
        _ => Err(ScraperError::ParseError(format!(
            "unknown rarity value: {value}"
        ))),
    }
}

fn parse_base_stats(value: &Value) -> ScraperResult<BaseStats> {
    let arr = value
        .as_array()
        .ok_or_else(|| ScraperError::ParseError("base_stats is not an array".into()))?;

    if arr.len() < 5 {
        return Err(ScraperError::ParseError(
            "base_stats has fewer than 5 elements".into(),
        ));
    }

    let mut stats = arr.iter().map(|v| {
        v.as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| ScraperError::ParseError("base_stats element is not a number".into()))
    });

    Ok(BaseStats {
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
        _ => Err(ScraperError::ParseError(format!(
            "unknown aptitude level: {value}"
        ))),
    }
}

fn parse_aptitudes(value: &Value) -> ScraperResult<Aptitudes> {
    let arr = value
        .as_array()
        .ok_or_else(|| ScraperError::ParseError("aptitude is not an array".into()))?;

    if arr.len() < 10 {
        return Err(ScraperError::ParseError(
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

    let categories = [
        ("skills_unique", SkillAcquisition::Unique),
        ("skills_innate", SkillAcquisition::Innate),
        ("skills_awakening", SkillAcquisition::Awakening),
        ("skills_event", SkillAcquisition::Event),
    ];

    for (key, acquisition) in categories {
        if let Some(arr) = item[key].as_array() {
            for v in arr {
                let id = v.as_u64().map(|n| SkillId(n as u32)).ok_or_else(|| {
                    ScraperError::ParseError(format!("invalid skill id in {key}"))
                })?;
                skills.push(UmaSkill { id, acquisition });
            }
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_next_data(item: serde_json::Value) -> String {
        format!(
            r#"<html><head><script id="__NEXT_DATA__" type="application/json">{}</script></head></html>"#,
            serde_json::json!({
                "props": {
                    "pageProps": {
                        "itemData": item
                    }
                }
            })
        )
    }

    fn valid_item() -> serde_json::Value {
        serde_json::json!({
            "card_id": 102001,
            "name_en": "Seiun Sky",
            "title_en_gl": "[Reeling in the Big One]",
            "rarity": 3,
            "base_stats": [98, 98, 88, 83, 83],
            "aptitude": ["A", "G", "G", "C", "A", "A", "A", "B", "D", "E"],
            "skills_unique": [100201],
            "skills_innate": [200881, 201192],
            "skills_awakening": [201522],
            "skills_event": [200891, 200742]
        })
    }

    #[test]
    fn parses_valid_character_page() {
        let html = make_next_data(valid_item());
        let uma = parse_character_page(&html).unwrap();

        assert_eq!(uma.id, UmaId(102001));
        assert_eq!(uma.name, "Seiun Sky");
        assert_eq!(uma.subtitle, "[Reeling in the Big One]");
        assert!(matches!(uma.rarity, UmaRarity::SSR));
        assert_eq!(uma.base_stats.speed, 98);
        assert_eq!(uma.base_stats.stamina, 98);
        assert_eq!(uma.base_stats.power, 88);
        assert_eq!(uma.base_stats.guts, 83);
        assert_eq!(uma.base_stats.wit, 83);
        assert!(matches!(uma.aptitudes.surface.turf, AptitudeLevel::A));
        assert!(matches!(uma.aptitudes.surface.dirt, AptitudeLevel::G));
        assert!(matches!(uma.aptitudes.distance.short, AptitudeLevel::G));
        assert!(matches!(uma.aptitudes.distance.mile, AptitudeLevel::C));
        assert!(matches!(uma.aptitudes.strategy.front, AptitudeLevel::A));
        assert!(matches!(uma.aptitudes.strategy.end, AptitudeLevel::E));
        assert_eq!(uma.skill_list.len(), 6);
    }

    #[test]
    fn errors_on_missing_next_data() {
        let html = "<html><body>no script here</body></html>";
        let result = parse_character_page(html);
        assert!(matches!(result, Err(ScraperError::ParseError(_))));
    }

    #[test]
    fn errors_on_invalid_rarity() {
        let mut item = valid_item();
        item["rarity"] = serde_json::json!(99);
        let html = make_next_data(item);
        let result = parse_character_page(&html);
        assert!(matches!(result, Err(ScraperError::ParseError(_))));
    }

    #[test]
    fn errors_on_short_base_stats() {
        let mut item = valid_item();
        item["base_stats"] = serde_json::json!([98, 98, 88]);
        let html = make_next_data(item);
        let result = parse_character_page(&html);
        assert!(matches!(result, Err(ScraperError::ParseError(_))));
    }

    #[test]
    fn errors_on_short_aptitude() {
        let mut item = valid_item();
        item["aptitude"] = serde_json::json!(["A", "B", "C"]);
        let html = make_next_data(item);
        let result = parse_character_page(&html);
        assert!(matches!(result, Err(ScraperError::ParseError(_))));
    }

    #[test]
    fn handles_missing_skill_categories() {
        let mut item = valid_item();
        item.as_object_mut().unwrap().remove("skills_event");
        item.as_object_mut().unwrap().remove("skills_awakening");
        let html = make_next_data(item);
        let uma = parse_character_page(&html).unwrap();
        assert_eq!(uma.skill_list.len(), 3); // unique + innate only
    }
}
