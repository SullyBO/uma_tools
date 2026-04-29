use uma_db::db::Db as database;
use uma_scraper::{
    client::ScraperClient, skill_condition_type_parser::fetch_skill_condition_types,
    skill_parser::fetch_skill_roster, uma_parser::fetch_uma_roster,
};

pub async fn sync_skills(db: &database) {
    let client = ScraperClient::builder().build();

    let skills = match fetch_skill_roster(&client).await {
        Ok(skills) => skills,
        Err(e) => {
            log::error!("Failed to fetch/parse skill roster: {e}");
            return;
        }
    };

    if let Err(e) = db.upsert_all_skills(&skills).await {
        log::error!("Failed to upsert all skills: {e}");
    }
}

pub async fn sync_uma(db: &database) {
    let client = ScraperClient::builder().build();

    let umas = match fetch_uma_roster(&client).await {
        Ok(umas) => umas,
        Err(e) => {
            log::error!("Failed to fetch/parse character roster: {e}");
            return;
        }
    };

    if let Err(e) = db.upsert_all_uma(&umas).await {
        log::error!("Failed to upsert all uma: {e}");
    }
}

pub async fn sync_conditions(db: &database) {
    let client = ScraperClient::builder().build();

    let conditions = match fetch_skill_condition_types(&client).await {
        Ok(conditions) => conditions,
        Err(e) => {
            log::error!("Failed to fetch/parse skill conditions: {e}");
            return;
        }
    };

    match db.upsert_all_condition_types(&conditions).await {
        Ok(_) => log::info!("Conditions sync complete: {} upserted", conditions.len()),
        Err(e) => log::error!("Failed to upsert condition types: {e}"),
    }
}
