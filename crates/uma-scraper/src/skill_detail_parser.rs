use scraper::{ElementRef, Html, Selector};
use uma_core::{
    ids::SkillId,
    models::skill::{SkillCondition, SkillEffect, SkillEffectStat, SkillOperator},
};

#[derive(Debug)]
pub struct SkillDetailData {
    pub id: SkillId,
    pub effects: Vec<SkillEffect>,
}

/// Parse skill details page from the `https://umamusu.wiki/Game:Skills/{id}`
pub fn parse_skill_detail_page(html: &str, id: SkillId) -> Option<SkillDetailData> {
    let document = Html::parse_document(html);
    let effect_sel = Selector::parse("div.skill-effect-wrapper").unwrap();
    let table_sel = Selector::parse("table.wikitable").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let mut effects = Vec::new();
    let mut skipped = 0usize;

    let effect_els: Vec<_> = document.select(&effect_sel).collect();

    if effect_els.is_empty() {
        log::warn!("Skill {}: no skill-effect-wrapper found on page", id.0);
        return None;
    }

    for effect_el in effect_els {
        let tables: Vec<_> = effect_el.select(&table_sel).collect();
        if tables.len() < 2 {
            log::warn!(
                "Skill {}: effect block skipped, expected 2 tables but found {}",
                id.0,
                tables.len()
            );
            skipped += 1;
            continue;
        }

        let stats = parse_stats(&tables[0], &tr_sel, &th_sel, &td_sel);
        let conditions = parse_conditions_table(&tables[1], &tr_sel, &th_sel, &td_sel);

        effects.push(SkillEffect { stats, conditions });
    }

    log::info!(
        "Skill {}: {} effects parsed, {} skipped",
        id.0,
        effects.len(),
        skipped
    );

    if effects.is_empty() {
        None
    } else {
        Some(SkillDetailData { id, effects })
    }
}

fn parse_stats(
    table: &ElementRef,
    tr_sel: &Selector,
    th_sel: &Selector,
    td_sel: &Selector,
) -> Vec<SkillEffectStat> {
    let mut stats = Vec::new();
    for row in table.select(tr_sel) {
        let stat_key = match row.select(th_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => continue,
        };
        let stat_val = match row.select(td_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => continue,
        };
        if !stat_key.is_empty() && !stat_val.is_empty() {
            stats.push(SkillEffectStat { stat_key, stat_val });
        }
    }
    stats
}

fn parse_conditions_table(
    table: &ElementRef,
    tr_sel: &Selector,
    th_sel: &Selector,
    td_sel: &Selector,
) -> Vec<SkillCondition> {
    let mut conditions = Vec::new();
    let rows: Vec<_> = table.select(tr_sel).collect();
    let mut i = 0;

    while i < rows.len() {
        let row = rows[i];

        let th = match row.select(th_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => {
                i += 1;
                continue;
            }
        };

        let is_precondition = th.contains("Precondition");
        let tds: Vec<_> = row.select(td_sel).collect();

        let is_or = tds
            .first()
            .map(|td| td.text().collect::<String>().trim() == "OR")
            .unwrap_or(false);

        if is_or {
            // First branch is the third td in this row (th, OR, branch)
            if let Some(branch_td) = tds.get(1) {
                let text = branch_td.text().collect::<String>().trim().to_string();
                conditions.extend(parse_condition_text(&text, is_precondition));
            }
            // Remaining branches are in subsequent rows with no th
            i += 1;
            while i < rows.len() {
                let next_row = rows[i];
                if next_row.select(th_sel).next().is_some() {
                    break;
                }
                if let Some(branch_td) = next_row.select(td_sel).next() {
                    let text = branch_td.text().collect::<String>().trim().to_string();
                    conditions.extend(parse_condition_text(&text, is_precondition));
                }
                i += 1;
            }
        } else {
            if let Some(td) = tds.first() {
                let text = td.text().collect::<String>().trim().to_string();
                conditions.extend(parse_condition_text(&text, is_precondition));
            }
            i += 1;
        }
    }

    conditions
}

fn parse_condition_text(text: &str, is_precondition: bool) -> Vec<SkillCondition> {
    let mut result = Vec::new();
    for part in text.split("AND") {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match parse_single_condition(part, is_precondition) {
            Some(cond) => result.push(cond),
            None => log::warn!("Failed to parse condition: '{}'", part),
        }
    }
    result
}

fn parse_single_condition(s: &str, is_precondition: bool) -> Option<SkillCondition> {
    let operators = [
        ("!=", SkillOperator::NotEq),
        (">=", SkillOperator::GtEq),
        ("<=", SkillOperator::LtEq),
        ("==", SkillOperator::Eq),
        (">", SkillOperator::Gt),
        ("<", SkillOperator::Lt),
    ];
    for (op_str, op_variant) in operators {
        if let Some(idx) = s.find(op_str) {
            let cond_key = s[..idx].trim().to_string();
            let cond_val = s[idx + op_str.len()..].trim().to_string();
            if !cond_key.is_empty() && !cond_val.is_empty() {
                return Some(SkillCondition {
                    is_precondition,
                    cond_key,
                    operator: op_variant,
                    cond_val,
                    description: None,
                    example: None,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use uma_core::ids::SkillId;

    fn make_id() -> SkillId {
        SkillId(200702)
    }

    fn wrap(inner: &str) -> String {
        format!(
            r#"<html><body><div class="skill-info-container"><div>{}</div></div></body></html>"#,
            inner
        )
    }

    fn simple_effect(stats: &str, conditions: &str) -> String {
        format!(
            r#"<div class="skill-effect-wrapper fit-content">
                <table class="wikitable"><tbody>{}</tbody></table>
                <table class="wikitable"><tbody>{}</tbody></table>
            </div>"#,
            stats, conditions
        )
    }

    #[test]
    fn test_parses_single_effect_and_conditions() {
        let html = wrap(&simple_effect(
            r#"<tr><th>Duration</th><td>3.0s</td></tr>
               <tr><th>Acceleration</th><td>0.2</td></tr>"#,
            r#"<tr><th>Conditions</th><td>distance_type == 2 <b>AND</b> phase_random == 2 <b>AND</b> order_rate &gt; 50</td></tr>"#,
        ));

        let result = parse_skill_detail_page(&html, make_id()).unwrap();
        assert_eq!(result.effects.len(), 1);

        let effect = &result.effects[0];
        assert_eq!(effect.stats.len(), 2);
        assert_eq!(effect.stats[0].stat_key, "Duration");
        assert_eq!(effect.stats[0].stat_val, "3.0s");
        assert_eq!(effect.stats[1].stat_key, "Acceleration");
        assert_eq!(effect.stats[1].stat_val, "0.2");

        assert_eq!(effect.conditions.len(), 3);
        assert!(!effect.conditions[0].is_precondition);
        assert_eq!(effect.conditions[0].cond_key, "distance_type");
        assert_eq!(effect.conditions[2].cond_key, "order_rate");
    }

    #[test]
    fn test_parses_preconditions_and_conditions() {
        let html = wrap(&simple_effect(
            r#"<tr><th>Duration</th><td>5.0s</td></tr>
               <tr><th>Target Speed</th><td>0.45</td></tr>"#,
            r#"<tr><th>Preconditions</th><td>phase == 1 <b>AND</b> blocked_side_continuetime &gt;= 2</td></tr>
               <tr><th>Conditions</th><td>is_finalcorner == 1 <b>AND</b> order &gt;= 2</td></tr>"#,
        ));

        let result = parse_skill_detail_page(&html, make_id()).unwrap();
        assert_eq!(result.effects.len(), 1);

        let effect = &result.effects[0];
        let preconditions: Vec<_> = effect
            .conditions
            .iter()
            .filter(|c| c.is_precondition)
            .collect();
        let conditions: Vec<_> = effect
            .conditions
            .iter()
            .filter(|c| !c.is_precondition)
            .collect();

        assert_eq!(preconditions.len(), 2);
        assert_eq!(conditions.len(), 2);
        assert_eq!(preconditions[0].cond_key, "phase");
        assert_eq!(conditions[0].cond_key, "is_finalcorner");
    }

    #[test]
    fn test_parses_multiple_effects() {
        let effect1 = simple_effect(
            r#"<tr><th>Duration</th><td>5.0s</td></tr>
               <tr><th>Target Speed</th><td>0.45</td></tr>"#,
            r#"<tr><th>Conditions</th><td>is_finalcorner == 1 <b>AND</b> order &gt;= 2</td></tr>"#,
        );
        let effect2 = simple_effect(
            r#"<tr><th>Duration</th><td>5.0s</td></tr>
               <tr><th>Target Speed</th><td>0.35</td></tr>"#,
            r#"<tr><th>Conditions</th><td>is_finalcorner == 1 <b>AND</b> order &gt;= 2</td></tr>"#,
        );
        let html = wrap(&format!("{}{}", effect1, effect2));

        let result = parse_skill_detail_page(&html, make_id()).unwrap();
        assert_eq!(result.effects.len(), 2);
        assert_eq!(result.effects[0].stats[1].stat_val, "0.45");
        assert_eq!(result.effects[1].stats[1].stat_val, "0.35");
    }

    #[test]
    fn test_parses_or_conditions() {
        let html = wrap(&format!(
            r#"<div class="skill-effect-wrapper fit-content">
                <table class="wikitable"><tbody>
                    <tr><th>Duration</th><td>5.0s</td></tr>
                    <tr><th>Target Speed</th><td>0.35</td></tr>
                    <tr><th>Acceleration</th><td>0.1</td></tr>
                </tbody></table>
                <table class="wikitable"><tbody>
                    <tr>
                        <th rowspan="2">Conditions</th>
                        <td rowspan="2"><b>OR</b></td>
                        <td>distance_rate &gt;= 50 <b>AND</b> corner == 0 <b>AND</b> order_rate &gt;= 70</td>
                    </tr>
                    <tr>
                        <td>distance_rate &gt;= 50 <b>AND</b> corner == 0 <b>AND</b> order_rate &lt;= 30</td>
                    </tr>
                </tbody></table>
            </div>"#
        ));

        let result = parse_skill_detail_page(&html, make_id()).unwrap();
        assert_eq!(result.effects.len(), 1);

        let effect = &result.effects[0];
        assert_eq!(effect.stats.len(), 3);
        assert_eq!(effect.conditions.len(), 6);
    }

    #[test]
    fn test_returns_none_for_no_effects() {
        let html = "<html><body><div>nothing here</div></body></html>";
        let result = parse_skill_detail_page(html, make_id());
        assert!(result.is_none());
    }

    #[test]
    fn test_skips_effect_missing_second_table() {
        let html = wrap(
            r#"<div class="skill-effect-wrapper fit-content">
                <table class="wikitable"><tbody>
                    <tr><th>Duration</th><td>3.0s</td></tr>
                </tbody></table>
            </div>"#,
        );
        let result = parse_skill_detail_page(&html, make_id());
        assert!(result.is_none());
    }
}
