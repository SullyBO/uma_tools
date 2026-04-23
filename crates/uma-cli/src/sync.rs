use uma_core::{ids::SkillId, models::skill::SkillEffect};
use uma_db::db::Db as database;
use uma_scraper::{
    client::ScraperClient, skill_detail_parser::parse_skill_detail_page,
    skill_parser::parse_skill_table, uma_list_parser::parse_uma_index,
    uma_parser::parse_all_characters,
};

pub async fn sync_skills(db: &database) {
    let client = ScraperClient::builder().build();

    let index_html = client
        .fetch("https://umamusu.wiki/Game:List_of_Skills")
        .await
        .expect("failed to fetch skills index");
    let skills = parse_skill_table(&index_html);

    let urls: Vec<String> = skills
        .iter()
        .map(|s| format!("https://umamusu.wiki/Game:Skills/{}", s.id.0))
        .collect();
    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    let detail_htmls = client.fetch_all(&url_refs).await;

    let mut pairs: Vec<_> = Vec::new();
    for (skill, result) in skills.into_iter().zip(detail_htmls) {
        match result {
            Ok(html) => match parse_skill_detail_page(&html, skill.id) {
                Some(detail) => pairs.push((skill, detail.effects)),
                None => {
                    log::warn!("No detail found for skill {}", skill.id.0);
                    pairs.push((skill, Vec::<SkillEffect>::new()));
                }
            },
            Err(e) => {
                log::warn!("Failed to fetch detail for skill {}: {e}", skill.id.0);
                pairs.push((skill, Vec::<SkillEffect>::new()));
            }
        }
    }

    db.upsert_all_skills(&pairs)
        .await
        .expect("failed to upsert skills");
}

pub async fn sync_characters(db: &database) {
    let client = ScraperClient::builder().build();

    let index_html = client
        .fetch("https://umamusu.wiki/Game:List_of_Trainees")
        .await
        .expect("failed to fetch trainee index");
    let entries = parse_uma_index(&index_html);

    let umas = parse_all_characters(&client, &entries).await;

    for result in umas {
        match result {
            Ok(uma) => {
                if let Err(e) = db.upsert_uma_full(&uma).await {
                    log::warn!("Failed to upsert uma {}: {e}", uma.id.0);
                }
            }
            Err(e) => log::warn!("Failed to parse character: {e}"),
        }
    }
}

pub async fn sync_skill_details(db: &database) {
    let skill_ids = db
        .get_all_skill_ids()
        .await
        .expect("failed to fetch skill ids");
    let client = ScraperClient::builder().build();

    let urls: Vec<String> = skill_ids
        .iter()
        .map(|id| format!("https://umamusu.wiki/Game:Skills/{}", id.0))
        .collect();
    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    let htmls = client.fetch_all(&url_refs).await;

    let mut pairs: Vec<(SkillId, Vec<SkillEffect>)> = Vec::new();
    for (id, result) in skill_ids.into_iter().zip(htmls) {
        match result {
            Ok(html) => match parse_skill_detail_page(&html, id) {
                Some(detail) => pairs.push((id, detail.effects)),
                None => log::warn!("No detail found for skill {}", id.0),
            },
            Err(e) => log::warn!("Failed to fetch detail for skill {}: {e}", id.0),
        }
    }

    db.upsert_all_skill_details(&pairs)
        .await
        .expect("failed to upsert skill details");
}
