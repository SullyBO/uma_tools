use log::{debug, info, warn};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use uma_core::{
    ids::SkillId,
    models::skill::{Skill, SkillCategory, SkillRarity},
};

/// Parses the skills table page from the umamusume wiki
pub fn parse_skill_table(html: &str) -> Vec<Skill> {
    let document = Html::parse_document(html);
    let mut skills = Vec::new();
    let mut total_rows = 0;
    let mut skip_reasons: HashMap<SkipReason, usize> = HashMap::new();

    let rarity_sections = [
        ("Common_Skills", SkillRarity::Normal),
        ("Rare_Skills", SkillRarity::Rare),
        ("Unique_Skills", SkillRarity::Unique),
    ];

    for (heading_id, rarity) in rarity_sections {
        if let Some(table) = find_table_after_heading(&document, heading_id) {
            let row_sel = Selector::parse("tbody tr").unwrap();
            for row in table.select(&row_sel) {
                total_rows += 1;
                match parse_skill_row(&row, rarity) {
                    ParseRowResult::Parsed(skill) => skills.push(skill),
                    ParseRowResult::Skipped(reason) => {
                        *skip_reasons.entry(reason).or_insert(0) += 1;
                    }
                }
            }
        } else {
            warn!("Section '{heading_id}' not found in page");
        }
    }

    let total_skipped: usize = skip_reasons.values().sum();

    info!(
        "Parsing complete: {} skills parsed, {} skipped out of {} total rows",
        skills.len(),
        total_skipped,
        total_rows
    );

    if !skip_reasons.is_empty() {
        info!("Skip reasons:");
        for (reason, count) in &skip_reasons {
            info!("  {}: {count}", reason.as_str());
        }
    }

    skills
}

enum ParseRowResult {
    Parsed(Skill),
    Skipped(SkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SkipReason {
    NoIconCell,
    NoIconId,
    NoNameCell,
    JpOnly,
    NoName,
    NoSkillId,
    NoDescription,
    NoSpCost,
    NoEvalPoints,
    UnknownIcon,
}

impl SkipReason {
    fn as_str(&self) -> &'static str {
        match self {
            SkipReason::NoIconCell => "no icon cell",
            SkipReason::NoIconId => "could not extract icon_id",
            SkipReason::NoNameCell => "no name cell",
            SkipReason::JpOnly => "JP-only skill",
            SkipReason::NoName => "could not extract name",
            SkipReason::NoSkillId => "could not extract skill_id",
            SkipReason::NoDescription => "no description cell",
            SkipReason::NoSpCost => "could not parse sp_cost",
            SkipReason::NoEvalPoints => "could not parse eval_points",
            SkipReason::UnknownIcon => "unknown icon_id",
        }
    }
}

fn find_table_after_heading<'a>(document: &'a Html, heading_id: &str) -> Option<ElementRef<'a>> {
    let span_sel = Selector::parse(&format!("span#{}", heading_id)).unwrap();
    let table_sel = Selector::parse("table").unwrap();

    let heading_span = document.select(&span_sel).next()?;
    let heading_node_id = heading_span.parent()?.id();

    document
        .select(&table_sel)
        .find(|table| table.id() > heading_node_id)
}

fn parse_skill_row(row: &ElementRef, rarity: SkillRarity) -> ParseRowResult {
    macro_rules! try_or_skip {
        ($expr:expr, $reason:expr) => {
            match $expr {
                Some(v) => v,
                None => {
                    debug!("Skipping row: {}", $reason.as_str());
                    return ParseRowResult::Skipped($reason);
                }
            }
        };
    }

    let td_sel = Selector::parse("td").unwrap();
    let mut tds = row.select(&td_sel);

    let icon_td = try_or_skip!(tds.next(), SkipReason::NoIconCell);
    let icon_id = try_or_skip!(extract_icon_id(&icon_td), SkipReason::NoIconId);
    let name_td = try_or_skip!(tds.next(), SkipReason::NoNameCell);

    if name_td
        .select(&Selector::parse("sup").unwrap())
        .next()
        .is_some()
    {
        debug!("Skipping row: JP-only skill");
        return ParseRowResult::Skipped(SkipReason::JpOnly);
    }

    let name = try_or_skip!(
        name_td
            .select(&Selector::parse("b a").unwrap())
            .next()
            .map(|a| a.text().collect::<String>()),
        SkipReason::NoName
    );
    let id: SkillId = try_or_skip!(extract_skill_id(&name_td), SkipReason::NoSkillId);
    let description = try_or_skip!(
        tds.next()
            .map(|td| td.text().collect::<String>().trim().to_string()),
        SkipReason::NoDescription
    );
    let sp_cost = try_or_skip!(
        tds.next()
            .and_then(|td| td.text().collect::<String>().trim().parse::<u32>().ok()),
        SkipReason::NoSpCost
    );
    let eval_points = try_or_skip!(
        tds.next()
            .and_then(|td| td.text().collect::<String>().trim().parse::<u32>().ok()),
        SkipReason::NoEvalPoints
    );
    let category = try_or_skip!(icon_id_to_category(icon_id), SkipReason::UnknownIcon);

    debug!("Parsed skill '{name}' (id={id:?}, rarity={rarity:?}, category={category:?})");

    ParseRowResult::Parsed(Skill {
        id,
        name,
        ingame_description: description,
        category,
        rarity,
        sp_cost,
        eval_points,
    })
}

fn extract_icon_id(td: &ElementRef) -> Option<u32> {
    let img_sel = Selector::parse("img").unwrap();
    let src = td.select(&img_sel).next()?.value().attr("src")?;
    // src looks like: /w/thumb.php?f=Game_Skill_Icon_20013.png&width=48
    let filename = src
        .split('?')
        .nth(1)?
        .split('&')
        .find(|s| s.starts_with("f="))?
        .trim_start_matches("f=Game_Skill_Icon_")
        .trim_end_matches(".png");
    filename.parse().ok()
}

fn extract_skill_id(td: &ElementRef) -> Option<SkillId> {
    let a_sel = Selector::parse("b a").unwrap();
    let href = td.select(&a_sel).next()?.value().attr("href")?;
    // href looks like: /Game:Skills/200011
    href.split('/').last()?.parse::<u32>().ok().map(SkillId)
}

fn icon_id_to_category(icon_id: u32) -> Option<SkillCategory> {
    let category = match icon_id {
        10011 => SkillCategory::Green, // Speed
        10012 => SkillCategory::Green, // Gold Speed
        10021 => SkillCategory::Green, // Stamina
        10022 => SkillCategory::Green, // Gold Stamina
        10031 => SkillCategory::Green, // Power
        10032 => SkillCategory::Green, // Gold Power
        10041 => SkillCategory::Green, // Guts
        10051 => SkillCategory::Green, // Wit
        10052 => SkillCategory::Green, // Gold Wit
        10061 => SkillCategory::Green, // Lucky seven
        10062 => SkillCategory::Green, // Super lucky seven
        40012 => SkillCategory::Green, // Runaway
        20021 => SkillCategory::Recovery,
        20022 => SkillCategory::Recovery,
        20023 => SkillCategory::Recovery,
        20011 => SkillCategory::Velocity,
        20012 => SkillCategory::Velocity,
        20013 => SkillCategory::Velocity,
        20041 => SkillCategory::Acceleration,
        20042 => SkillCategory::Acceleration,
        20043 => SkillCategory::Acceleration,
        20051 => SkillCategory::Movement,
        20052 => SkillCategory::Movement,
        20061 => SkillCategory::Gate,
        20062 => SkillCategory::Gate,
        20091 => SkillCategory::Vision,
        20092 => SkillCategory::Vision,
        30011 => SkillCategory::SpeedDebuff,
        30012 => SkillCategory::SpeedDebuff,
        30021 => SkillCategory::AccelDebuff,
        30022 => SkillCategory::AccelDebuff,
        30041 => SkillCategory::FrenzyDebuff,
        30051 => SkillCategory::StaminaDrain,
        30052 => SkillCategory::StaminaDrain,
        30071 => SkillCategory::VisionDebuff,
        30072 => SkillCategory::VisionDebuff,
        10014 => SkillCategory::Purple,     // Speed
        10024 => SkillCategory::Purple,     // Stamina
        10034 => SkillCategory::Purple,     // Power
        10044 => SkillCategory::Purple,     // Guts
        10054 => SkillCategory::Purple,     // Wit
        20064 => SkillCategory::Purple,     // Gate
        20014 => SkillCategory::Purple,     // Velocity
        20015 => SkillCategory::Purple,     // Gold Velocity
        20044 => SkillCategory::Purple,     // Accel
        20045 => SkillCategory::Purple,     // Gold Accel
        20024 => SkillCategory::Purple,     // Recovery
        20101 => SkillCategory::Scenario,   // Ignited SPD
        20102 => SkillCategory::Scenario,   // Burning SPD
        20121 => SkillCategory::Scenario,   // Ignited PWR
        20122 => SkillCategory::Scenario,   // Burning PWR
        20111 => SkillCategory::Scenario,   // Ignited STA
        20112 => SkillCategory::Scenario,   // Burning STA
        20131 => SkillCategory::Scenario,   // Ignited WIT
        20132 => SkillCategory::Scenario,   // Burning WIT
        20141 => SkillCategory::Scenario,   // Glittering Star
        20142 => SkillCategory::Scenario,   // Radiant Star
        2010010 => SkillCategory::Velocity, // Best in Japan
        other => {
            warn!("Unknown icon_id: {other}, skill will be skipped");
            return None;
        }
    };
    debug!("Mapped icon_id {icon_id} to {category:?}");
    Some(category)
}

#[cfg(test)]
mod tests {

    use uma_core::ids::SkillId;

    use crate::skill_parser::parse_skill_table;

    // Minimal HTML that mirrors the real wiki structure:
    // - A heading with the expected span id
    // - A table after it with one valid skill row
    fn make_page(section_id: &str, icon_id: u32, skill_id: u32) -> String {
        format!(
            r#"
            <h2><span id="{section_id}"></span></h2>
            <table>
              <tbody>
                <tr>
                  <td>
                    <img src="/w/thumb.php?f=Game_Skill_Icon_{icon_id}.png&width=48"/>
                  </td>
                  <td>
                    <b><a href="/Game:Skills/{skill_id}">Speed Boost</a></b>
                  </td>
                  <td>Increases speed for a short time.</td>
                  <td>150</td>
                  <td>120</td>
                </tr>
              </tbody>
            </table>
            "#
        )
    }

    #[test]
    fn parses_skill_name_and_id() {
        let html = make_page("Common_Skills", 20013, 200011);
        let skills = parse_skill_table(&html);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "Speed Boost");
        assert_eq!(skills[0].id, SkillId(200011));
    }

    #[test]
    fn parses_sp_cost_and_eval_points() {
        let html = make_page("Common_Skills", 20013, 200011);
        let skills = parse_skill_table(&html);
        assert_eq!(skills[0].sp_cost, 150);
        assert_eq!(skills[0].eval_points, 120);
    }

    #[test]
    fn parses_ingame_description() {
        let html = make_page("Common_Skills", 20013, 200011);
        let skills = parse_skill_table(&html);
        assert_eq!(
            skills[0].ingame_description,
            "Increases speed for a short time."
        );
    }

    #[test]
    fn skips_row_missing_icon_img() {
        let html = r#"
            <h2><span id="Common_Skills"></span></h2>
            <table>
              <tbody>
                <tr>
                  <td><!-- no img here --></td>
                  <td><b><a href="/Game:Skills/200011">Speed Boost</a></b></td>
                  <td>Increases speed for a short time.</td>
                  <td>150</td>
                  <td>120</td>
                </tr>
              </tbody>
            </table>
        "#;
        let skills = parse_skill_table(html);
        assert!(skills.is_empty());
    }

    #[test]
    fn skips_row_with_non_numeric_sp_cost() {
        let html = r#"
            <h2><span id="Common_Skills"></span></h2>
            <table>
              <tbody>
                <tr>
                  <td><img src="/w/thumb.php?f=Game_Skill_Icon_20013.png&width=48"/></td>
                  <td><b><a href="/Game:Skills/200011">Speed Boost</a></b></td>
                  <td>Increases speed for a short time.</td>
                  <td>???</td>
                  <td>120</td>
                </tr>
              </tbody>
            </table>
        "#;
        let skills = parse_skill_table(html);
        assert!(skills.is_empty());
    }

    #[test]
    fn ignores_table_before_heading() {
        // A table that comes BEFORE the heading span should not be picked up
        let html = r#"
            <table>
              <tbody>
                <tr>
                  <td><img src="/w/thumb.php?f=Game_Skill_Icon_20013.png&width=48"/></td>
                  <td><b><a href="/Game:Skills/200011">Speed Boost</a></b></td>
                  <td>Increases speed for a short time.</td>
                  <td>150</td>
                  <td>120</td>
                </tr>
              </tbody>
            </table>
            <h2><span id="Common_Skills"></span></h2>
        "#;
        let skills = parse_skill_table(html);
        assert!(skills.is_empty());
    }

    #[test]
    fn parses_multiple_rarity_sections() {
        let html = r#"
            <h2><span id="Common_Skills"></span></h2>
            <table>
              <tbody>
                <tr>
                  <td><img src="/w/thumb.php?f=Game_Skill_Icon_20013.png&width=48"/></td>
                  <td><b><a href="/Game:Skills/200011">Speed Boost</a></b></td>
                  <td>Increases speed.</td>
                  <td>150</td>
                  <td>120</td>
                </tr>
              </tbody>
            </table>
            <h2><span id="Rare_Skills"></span></h2>
            <table>
              <tbody>
                <tr>
                  <td><img src="/w/thumb.php?f=Game_Skill_Icon_30021.png&width=48"/></td>
                  <td><b><a href="/Game:Skills/300042">Burst</a></b></td>
                  <td>Greatly increases speed.</td>
                  <td>200</td>
                  <td>250</td>
                </tr>
              </tbody>
            </table>
        "#;
        let skills = parse_skill_table(html);
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "Speed Boost");
        assert_eq!(skills[1].name, "Burst");
    }
}
