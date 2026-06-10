use crate::error::{ScraperError, ScraperResult};
use serde_json::Value;
use uma_core::{
    ids::SkillId,
    models::skill::{Condition, Duration, Effect, EffectType, Operator},
};

pub fn parse_effect(cg: &Value, skill_id: SkillId) -> ScraperResult<Effect> {
    let base_time = cg["base_time"]
        .as_i64()
        .ok_or_else(|| ScraperError::MissingField(format!("base_time in skill {}", skill_id.0)))?;

    let duration = if base_time == -1 {
        Duration::Infinite
    } else {
        Duration::Timed(base_time as f32 / 10000.0)
    };

    let scaling = parse_scaling(cg);

    let effects = cg["effects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| parse_effect_type(e))
        .collect();

    let conditions = parse_condition_string(cg["condition"].as_str().unwrap_or(""), skill_id)?;
    let preconditions =
        parse_condition_string(cg["precondition"].as_str().unwrap_or(""), skill_id)?;

    Ok(Effect {
        duration,
        scaling,
        effects,
        conditions,
        preconditions,
    })
}

pub fn parse_scaling(cg: &Value) -> Option<String> {
    if let Some(time_scale) = cg["time_scale"].as_u64() {
        return match time_scale {
            2 => Some("Duration scales linearly 0.8–1.6× with distance from first place".into()),
            3 => Some("Duration scales with remaining HP".into()),
            4 => Some("Each overtake (up to 3) extends base duration by 1 second".into()),
            _ => None,
        };
    }

    let effects = cg["effects"].as_array()?;
    for effect in effects {
        if let Some(value_scale) = effect["value_scale"].as_u64() {
            return match value_scale {
                2 => Some("Multiplier scales with number of skills learned (1 + 0.01 × skills, max 1.2 at 20)".into()),
                3 => Some("Multiplier scales with combined team Speed stat".into()),
                4 => Some("Multiplier scales with combined team Stamina stat".into()),
                5 => Some("Multiplier scales with combined team Power stat".into()),
                6 => Some("Multiplier scales with combined team Guts stat".into()),
                7 => Some("Multiplier scales with combined team Wit stat".into()),
                8 => Some("Multiplier is random (60%: none, 30%: +0.02, 10%: +0.04)".into()),
                10 => Some("Multiplier scales with race victories during training".into()),
                11 => Some("Multiplier scales with overtakes during Late-Race corners".into()),
                12 => Some("Multiplier scales with fan count".into()),
                14 => Some("Multiplier scales with green skills activated in the race".into()),
                19 => Some("Each overtake (up to 3) grants additional acceleration for the remaining duration".into()),
                20 => Some("Each skill activated while this is active (up to 3) grants additional speed for the remaining duration".into()),
                22 | 23 => Some("Multiplier scales with base Speed stat".into()),
                24 => Some("Multiplier scales with cumulative Overseas Aptitude level".into()),
                _ => None,
            };
        }
    }

    None
}

fn parse_effect_type(e: &Value) -> Option<EffectType> {
    let type_id = e["type"].as_u64()?;
    let raw = e["value"].as_f64().unwrap_or(0.0);
    let scale = |divisor: f64| (raw / divisor) as f32;

    match type_id {
        1 => Some(EffectType::SpeedUp(scale(10000.0))),
        2 => Some(EffectType::StaminaUp(scale(10000.0))),
        3 => Some(EffectType::PowerUp(scale(10000.0))),
        4 => Some(EffectType::GutsUp(scale(10000.0))),
        5 => Some(EffectType::WitUp(scale(10000.0))),
        6 => Some(EffectType::RunawaySkill),
        8 => Some(EffectType::FieldOfViewUp(scale(10000.0))),
        9 => {
            let has_target = e["target"].is_number();
            if has_target {
                Some(EffectType::StaminaDrain(scale(10000.0).abs()))
            } else {
                Some(EffectType::StaminaRecovery(scale(10000.0)))
            }
        }
        10 => Some(EffectType::StartReactionImprovement(scale(10000.0))),
        13 => Some(EffectType::RushTimeIncrease(scale(10000.0))),
        14 => Some(EffectType::StartDelayAdded(scale(10000.0))),
        21 => Some(EffectType::CurrentSpeedDown(scale(10000.0))),
        22 => Some(EffectType::CurrentSpeedUp(scale(10000.0))),
        27 => Some(EffectType::TargetSpeedUp(scale(10000.0))),
        28 => Some(EffectType::LaneChangeSpeed(scale(1000.0))),
        29 => Some(EffectType::RushChanceDecrease(scale(10000.0))),
        31 => Some(EffectType::AccelerationUp(scale(10000.0))),
        32 => Some(EffectType::AllStatsUp(scale(10000.0))),
        35 => Some(EffectType::ChangeLane(scale(100.0))),
        37 => Some(EffectType::UseRandomRareSkills(scale(10000.0))),
        38 => Some(EffectType::DebuffImmunity),
        41 => Some(EffectType::ActivateRelatedSkillsOnAllUma),
        42 => Some(EffectType::EvolvedSkillDurationUp(scale(1000.0))),
        48 => Some(EffectType::ZenkaiSpurtAcceleration(scale(10000.0))),
        _ => None,
    }
}

fn parse_condition_string(s: &str, skill_id: SkillId) -> ScraperResult<Vec<Condition>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    let mut conditions = Vec::new();

    for (or_idx, or_group) in s.split('@').enumerate() {
        for (and_idx, part) in or_group.split('&').enumerate() {
            let (cond_key, operator, cond_val) =
                parse_condition_operator(part).ok_or_else(|| {
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

fn parse_condition_operator(s: &str) -> Option<(String, Operator, String)> {
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

    fn make_cg(extra: serde_json::Value) -> serde_json::Value {
        let mut base = serde_json::json!({
            "base_time": 50000,
            "effects": [{"type": 27, "value": 2500}],
            "condition": "phase>=2",
            "precondition": ""
        });
        if let (Some(obj), Some(extra_obj)) = (base.as_object_mut(), extra.as_object()) {
            obj.extend(extra_obj.clone());
        }
        base
    }

    #[test]
    fn parses_timed_duration() {
        let cg = make_cg(serde_json::json!({}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(matches!(effect.duration, Duration::Timed(v) if (v - 5.0).abs() < f32::EPSILON));
    }

    #[test]
    fn parses_infinite_duration() {
        let cg = make_cg(serde_json::json!({"base_time": -1}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(matches!(effect.duration, Duration::Infinite));
    }

    #[test]
    fn no_scaling_when_absent() {
        let cg = make_cg(serde_json::json!({}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.scaling.is_none());
    }

    #[test]
    fn parses_time_scale() {
        let cg = make_cg(serde_json::json!({"time_scale": 2}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.scaling.is_some());
        assert!(
            effect
                .scaling
                .unwrap()
                .contains("distance from first place")
        );
    }

    #[test]
    fn parses_value_scale() {
        let cg = make_cg(serde_json::json!({
            "effects": [{"type": 27, "value": 2500, "value_scale": 14}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.scaling.is_some());
        assert!(effect.scaling.unwrap().contains("green skills"));
    }

    #[test]
    fn time_scale_takes_priority_over_value_scale() {
        let cg = make_cg(serde_json::json!({
            "time_scale": 2,
            "effects": [{"type": 27, "value": 2500, "value_scale": 14}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(
            effect
                .scaling
                .unwrap()
                .contains("distance from first place")
        );
    }

    #[test]
    fn unknown_time_scale_returns_none() {
        let cg = make_cg(serde_json::json!({"time_scale": 99}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.scaling.is_none());
    }

    #[test]
    fn unknown_value_scale_returns_none() {
        let cg = make_cg(serde_json::json!({
            "effects": [{"type": 27, "value": 2500, "value_scale": 99}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.scaling.is_none());
    }

    #[test]
    fn skips_unknown_effect_types() {
        let cg = make_cg(serde_json::json!({
            "effects": [{"type": 9999, "value": 100}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(effect.effects.is_empty());
    }

    #[test]
    fn parses_conditions() {
        let cg = make_cg(serde_json::json!({"condition": "distance_rate>=50&order_rate>50"}));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert_eq!(effect.conditions.len(), 2);
        assert_eq!(effect.conditions[0].cond_key, "distance_rate");
    }

    #[test]
    fn parses_or_conditions() {
        let cg = make_cg(serde_json::json!({
            "condition": "distance_rate>=50&order<=3@distance_rate>=50&order_rate<=50"
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        let conditions = &effect.conditions;
        assert_eq!(conditions.len(), 4);
        assert!(!conditions[0].is_or);
        assert!(!conditions[1].is_or);
        assert!(conditions[2].is_or);
        assert!(!conditions[3].is_or);
    }

    #[test]
    fn errors_on_missing_base_time() {
        let mut cg = make_cg(serde_json::json!({}));
        cg.as_object_mut().unwrap().remove("base_time");
        assert!(matches!(
            parse_effect(&cg, SkillId(1)),
            Err(ScraperError::MissingField(_))
        ));
    }

    #[test]
    fn errors_on_malformed_condition() {
        let cg = make_cg(serde_json::json!({"condition": "notacondition"}));
        assert!(matches!(
            parse_effect(&cg, SkillId(1)),
            Err(ScraperError::InvalidCondition(_))
        ));
    }

    #[test]
    fn parses_stamina_drain() {
        let cg = make_cg(serde_json::json!({
            "effects": [{"type": 9, "target": 9, "target_details": 5, "value": -100}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(matches!(effect.effects[0], EffectType::StaminaDrain(v) if v > 0.0));
    }

    #[test]
    fn parses_stamina_recovery() {
        let cg = make_cg(serde_json::json!({
            "effects": [{"type": 9, "value": 350}]
        }));
        let effect = parse_effect(&cg, SkillId(1)).unwrap();
        assert!(matches!(effect.effects[0], EffectType::StaminaRecovery(_)));
    }
}
