use crate::error::{ScraperError, ScraperResult};
use log::warn;
use serde_json::Value;

/// Collects all value fields from a unique effect JSON object into an ordered Vec.
///
/// Unique effects encode their parameters as sequential keys: `value`, `value_1`,
/// `value_2`, `value_3`, `value_4`. Not all keys are present on every effect —
/// only keys that exist in the object are included, preserving positional order.
pub fn collect_effect_values(effect: &Value) -> Vec<i64> {
    let mut values = Vec::new();
    if let Some(v) = effect["value"].as_i64() {
        values.push(v);
    }
    if let Some(v) = effect["value_1"].as_i64() {
        values.push(v);
    }
    if let Some(v) = effect["value_2"].as_i64() {
        values.push(v);
    }
    if let Some(v) = effect["value_3"].as_i64() {
        values.push(v);
    }
    if let Some(v) = effect["value_4"].as_i64() {
        values.push(v);
    }
    values
}

pub fn parse_unique_effect(unique: &Value) -> ScraperResult<String> {
    let effects = unique["effects"]
        .as_array()
        .ok_or_else(|| ScraperError::MissingField("unique.effects".into()))?;

    if effects.is_empty() {
        return Err(ScraperError::InvalidShape(
            "unique effects array is empty".into(),
        ));
    }

    let is_conditional = effects
        .first()
        .and_then(|e| e["type"].as_u64())
        .map(|t| t == 101)
        .unwrap_or(false);

    let mut parts = Vec::new();

    for effect in effects {
        let type_id = effect["type"]
            .as_u64()
            .ok_or_else(|| ScraperError::MissingField("unique effect type".into()))?;

        let values = collect_effect_values(effect);

        match translate_unique_effect_type(type_id, &values) {
            Some(description) => parts.push(description),
            None => {
                return Err(ScraperError::UnknownValue(format!(
                    "unique effect type ID {type_id}"
                )));
            }
        }
    }

    let body = if is_conditional {
        parts.join(" ")
    } else {
        parts.join(", ")
    };

    Ok(body)
}

/// Translates a unique effect type ID and its values into a human-readable English description.
///
/// Effect type IDs fall into two categories:
/// - **Simple effects** (1-41): Direct stat bonuses or modifiers. `values[0]` is always
///   the effect magnitude.
/// - **Conditional effects** (101+): Complex game mechanics that depend on runtime state
///   such as bond gauge level, deck composition, or facility configuration. These use
///   multiple value slots whose meaning varies per type.
///
/// Type 101 is special — it wraps another effect type with a bond gauge condition.
/// `values[0]` is the threshold, `values[1]` is the nested effect type ID, and
/// `values[2..]` are that effect's parameters. This function recurses to resolve it.
///
/// Returns `None` for unknown type IDs, which causes the parent card to be dropped
/// during parsing. Add new type IDs here as they are identified.
fn translate_unique_effect_type(type_id: u64, values: &[i64]) -> Option<String> {
    let v0 = values.get(0).copied().unwrap_or(0);
    let v1 = values.get(1).copied().unwrap_or(0);
    let v2 = values.get(2).copied().unwrap_or(0);
    let v3 = values.get(3).copied().unwrap_or(0);
    let v4 = values.get(4).copied().unwrap_or(0);

    match type_id {
        1 => Some(format!("Friendship Bonus {v0}")),
        2 => Some(format!("Mood Effect {v0}")),
        3 => Some(format!("Speed Bonus {v0}")),
        4 => Some(format!("Stamina Bonus {v0}")),
        5 => Some(format!("Power Bonus {v0}")),
        6 => Some(format!("Guts Bonus {v0}")),
        7 => Some(format!("Wit Bonus {v0}")),
        8 => Some(format!("Training Effectiveness {v0}%")),
        9 => Some(format!("Initial Speed {v0}")),
        10 => Some(format!("Initial Stamina {v0}")),
        11 => Some(format!("Initial Power {v0}")),
        12 => Some(format!("Initial Guts {v0}")),
        13 => Some(format!("Initial Wit {v0}")),
        14 => Some(format!("Initial Friendship Gauge {v0}")),
        15 => Some(format!("Race Bonus {v0}%")),
        16 => Some(format!("Fan Bonus {v0}%")),
        17 => Some(format!("Hint Levels {v0}")),
        18 => Some(format!("Hint Frequency {v0}%")),
        19 => Some(format!("Specialty Priority {v0}")),
        25 => Some(format!("Event Recovery {v0}%")),
        26 => Some(format!("Event Effectiveness {v0}%")),
        27 => Some(format!("Failure Protection {v0}%")),
        28 => Some(format!("Energy Cost Reduction {v0}%")),
        30 => Some(format!("Skill Point Bonus {v0}")),
        31 => Some(format!("Wit Friendship Recovery {v0}")),
        32 => Some(format!("Initial Skill Points {v0}")),
        33 => Some(format!("Hint Quantity Bonus {v0}")),
        41 => Some(format!("All Stats Bonus {v0}")),
        101 => {
            let condition = if v0 == 100 {
                "when the bond gauge is full".to_string()
            } else {
                format!("when bond gauge is at least {v0}")
            };
            let effect = translate_unique_effect_type(v1 as u64, &values[2..]);
            effect.map(|e| format!("Gain {e} {condition}"))
        }
        102 => Some(format!(
            "When the bond gauge is {v0} or higher and this card is in a facility different from its type, gain increased Training Effectiveness {v1}"
        )),
        103 => Some(format!(
            "If there are at least {v0} different types of support cards in your deck, gain increased Training Effectiveness {v1}"
        )),
        104 => Some(format!(
            "Gain increased Training Effectiveness (1) per {v0} fans, up to {v1} for {}",
            v0 * v1
        )),
        105 => Some(format!(
            "Gain Initial Stat Up ({v0}), where Stat is the type of card, for every card in your support deck (Friend and Group types give ({v1}) to every stat)"
        )),
        106 => Some(format!(
            "Gain Friendship Bonus ({v2}) every time you do friendship training with this card, up to {v0} times for a total of ({})",
            v0 * v2
        )),
        107 => Some(format!(
            "The less energy you have, the more Friendship Bonus you'll gain (Max: {v3})"
        )),
        108 => Some(format!(
            "The more maximum energy you have, the higher Training Effectiveness you'll gain (Max: {v4})"
        )),
        109 => Some(
            "The higher the bond of all your support is, the higher Training Effectiveness you'll gain (Max: 20)".to_string()
        ),
        110 => Some(format!(
            "Gain Training Effectiveness ({v1}) for every support card in the same training facility"
        )),
        111 => Some(format!(
            "Gain Training Effectiveness ({v1}) per every level of the current training facility"
        )),
        112 => Some(format!(
            "{v0}% chance to make the current training fail rate zero"
        )),
        113 => Some(format!(
            "Gain Mood Effect ({v1}) when participating in a friendship training"
        )),
        114 => Some(format!(
            "The more energy you have, the more Training Effectiveness you'll gain (Max: {v2})"
        )),
        115 => Some(format!(
            "All support cards gain Initial Friendship Gauge ({v1})"
        )),
        116 => Some(format!(
            "Gain Training Effectiveness ({v2}) for every Speed-increasing skill, up to ({}) for {v3} skills",
            v2 * v3
        )),
        117 => Some(format!(
            "The higher combined facility level, the higher Training Effectiveness you'll gain (up to {v2})"
        )),
        118 => Some(format!(
            "When the bond gauge is {v1} or higher, this support can appear in 2 training facilities at once"
        )),
        119 => Some(format!(
            "When the bond gauge is {v2} or higher, all support cards in your deck are more likely to appear in training facilities"
        )),
        120 => Some(format!(
            "When the bond gauge is {v1} or higher, gain Stat Bonus, where Stat is the type of the card, for every card in your support deck (up to 2 per Stat); Friend and Group cards give skill points"
        )),
        121 => Some(format!(
            "All supports get {v0} bonus bond from training; {v1} if they're in the same facility as this card"
        )),
        122 => Some(format!(
            "Supports that train together with this one gain Specialty Priority ({v1}) for the next turn"
        )),
        unknown => {
            warn!("Unknown unique effect type ID: {unknown} - add to translation map");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unique_effect_non_conditional() {
        let unique = serde_json::json!({
            "level": 25,
            "effects": [
                { "type": 8, "value": 5 },
                { "type": 15, "value": 5 }
            ]
        });
        let result = parse_unique_effect(&unique).unwrap();
        assert_eq!(result, "Training Effectiveness 5%, Race Bonus 5%");
    }

    #[test]
    fn parse_unique_effect_conditional() {
        let unique = serde_json::json!({
            "level": 0,
            "effects": [
                { "type": 101, "value": 80, "value_1": 5, "value_2": 10 }
            ]
        });
        let result = parse_unique_effect(&unique).unwrap();
        assert_eq!(result, "Gain Power Bonus 10 when bond gauge is at least 80");
    }

    #[test]
    fn parse_unique_effect_full_bond_gauge() {
        let unique = serde_json::json!({
            "level": 0,
            "effects": [
                { "type": 101, "value": 100, "value_1": 8, "value_2": 15 }
            ]
        });
        let result = parse_unique_effect(&unique).unwrap();
        assert_eq!(
            result,
            "Gain Training Effectiveness 15% when the bond gauge is full"
        );
    }

    #[test]
    fn parse_unique_effect_empty_array_errors() {
        let unique = serde_json::json!({
            "level": 0,
            "effects": []
        });
        assert!(matches!(
            parse_unique_effect(&unique),
            Err(ScraperError::InvalidShape(_))
        ));
    }

    #[test]
    fn parse_unique_effect_unknown_type_errors() {
        let unique = serde_json::json!({
            "level": 0,
            "effects": [
                { "type": 999, "value": 5 }
            ]
        });
        assert!(matches!(
            parse_unique_effect(&unique),
            Err(ScraperError::UnknownValue(_))
        ));
    }
}
