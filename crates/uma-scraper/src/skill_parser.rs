use scraper::{ElementRef, Html, Selector};
use uma_core::models::skill::{Skill, SkillCategory, SkillRarity};

pub fn parse_skills_page(html: &str) -> Vec<Skill> {
    let document = Html::parse_document(html);
    let mut skills = Vec::new();

    let rarity_sections = [
        ("Common_Skills", SkillRarity::Normal),
        ("Rare_Skills", SkillRarity::Rare),
        ("Unique_Skills", SkillRarity::Unique),
    ];

    for (heading_id, rarity) in rarity_sections {
        if let Some(table) = find_table_after_heading(&document, heading_id) {
            let row_sel = Selector::parse("tbody tr").unwrap();
            for row in table.select(&row_sel) {
                if let Some(skill) = parse_skill_row(&row, rarity) {
                    skills.push(skill);
                }
            }
        }
    }

    skills
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

fn parse_skill_row(row: &ElementRef, rarity: SkillRarity) -> Option<Skill> {
    let td_sel = Selector::parse("td").unwrap();
    let mut tds = row.select(&td_sel);

    let icon_td = tds.next()?;
    println!("icon_td ok");
    let icon_id = extract_icon_id(&icon_td);
    println!("icon_id: {:?}", icon_id);
    let icon_id = icon_id?;

    let name_td = tds.next()?;
    println!("name_td ok");
    let name = name_td
        .select(&Selector::parse("b a").unwrap())
        .next()
        .map(|a| a.text().collect::<String>());
    println!("name: {:?}", name);
    let name = name?;

    let id = extract_skill_id(&name_td);
    println!("id: {:?}", id);
    let id = id?;

    let description = tds
        .next()
        .map(|td| td.text().collect::<String>().trim().to_string());
    println!("description: {:?}", description);
    let description = description?;

    let sp_cost = tds
        .next()
        .and_then(|td| td.text().collect::<String>().trim().parse::<u32>().ok());
    println!("sp_cost: {:?}", sp_cost);
    let sp_cost = sp_cost?;

    let eval_points = tds
        .next()
        .and_then(|td| td.text().collect::<String>().trim().parse::<u32>().ok());
    println!("eval_points: {:?}", eval_points);
    let eval_points = eval_points?;

    Some(Skill {
        id,
        name,
        ingame_description: description,
        category: icon_id_to_category(icon_id),
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

fn extract_skill_id(td: &ElementRef) -> Option<u32> {
    let a_sel = Selector::parse("b a").unwrap();
    let href = td.select(&a_sel).next()?.value().attr("href")?;
    // href looks like: /Game:Skills/200011
    href.split('/').last()?.parse().ok()
}

fn icon_id_to_category(icon_id: u32) -> SkillCategory {
    // TODO: map icon IDs to categories once we have the full list
    todo!("icon_id_to_category: {icon_id}")
}

#[cfg(test)]
mod tests {

    use crate::skill_parser::parse_skills_page;

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
        let skills = parse_skills_page(&html);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "Speed Boost");
        assert_eq!(skills[0].id, 200011);
    }

    #[test]
    fn parses_sp_cost_and_eval_points() {
        let html = make_page("Common_Skills", 20013, 200011);
        let skills = parse_skills_page(&html);
        assert_eq!(skills[0].sp_cost, 150);
        assert_eq!(skills[0].eval_points, 120);
    }

    #[test]
    fn parses_ingame_description() {
        let html = make_page("Common_Skills", 20013, 200011);
        let skills = parse_skills_page(&html);
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
        let skills = parse_skills_page(html);
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
        let skills = parse_skills_page(html);
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
        let skills = parse_skills_page(html);
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
        let skills = parse_skills_page(html);
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "Speed Boost");
        assert_eq!(skills[1].name, "Burst");
    }
}
