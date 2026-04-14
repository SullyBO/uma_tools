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
