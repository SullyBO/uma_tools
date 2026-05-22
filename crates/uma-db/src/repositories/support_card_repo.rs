use crate::types::{
    DbCardRarity, DbCardType, DbSupportSkillAcquisition, SupportCardEffectRow, SupportCardRow,
    SupportCardSkillRow,
};
use sqlx::PgPool;
use uma_core::models::support_card::{EffectValue, SupportCard};

fn effect_value_to_id(e: &EffectValue) -> i32 {
    match e {
        EffectValue::FriendshipBonus(_) => 1,
        EffectValue::MoodEffect(_) => 2,
        EffectValue::SpeedBonus(_) => 3,
        EffectValue::StaminaBonus(_) => 4,
        EffectValue::PowerBonus(_) => 5,
        EffectValue::GutsBonus(_) => 6,
        EffectValue::WitBonus(_) => 7,
        EffectValue::TrainingEffectiveness(_) => 8,
        EffectValue::InitialSpeed(_) => 9,
        EffectValue::InitialStamina(_) => 10,
        EffectValue::InitialPower(_) => 11,
        EffectValue::InitialGuts(_) => 12,
        EffectValue::InitialWit(_) => 13,
        EffectValue::InitialFriendshipGauge(_) => 14,
        EffectValue::RaceBonus(_) => 15,
        EffectValue::FanBonus(_) => 16,
        EffectValue::HintLevels(_) => 17,
        EffectValue::HintFrequency(_) => 18,
        EffectValue::SpecialtyPriority(_) => 19,
        EffectValue::MaxSpeed(_) => 20,
        EffectValue::MaxStamina(_) => 21,
        EffectValue::MaxPower(_) => 22,
        EffectValue::MaxGuts(_) => 23,
        EffectValue::MaxWit(_) => 24,
        EffectValue::EventRecovery(_) => 25,
        EffectValue::EventEffectiveness(_) => 26,
        EffectValue::FailureProtection(_) => 27,
        EffectValue::EnergyCostReduction(_) => 28,
        EffectValue::MinigameEffectiveness(_) => 29,
        EffectValue::SkillPointBonus(_) => 30,
        EffectValue::WitFriendshipRecovery(_) => 31,
        EffectValue::InitialSkillPoints(_) => 32,
        EffectValue::HintQuantityBonus(_) => 33,
        EffectValue::AllStatsBonus(_) => 41,
    }
}

fn effect_value_to_lb_values(e: &EffectValue) -> &uma_core::models::support_card::LbValues {
    match e {
        EffectValue::FriendshipBonus(v)
        | EffectValue::MoodEffect(v)
        | EffectValue::SpeedBonus(v)
        | EffectValue::StaminaBonus(v)
        | EffectValue::PowerBonus(v)
        | EffectValue::GutsBonus(v)
        | EffectValue::WitBonus(v)
        | EffectValue::TrainingEffectiveness(v)
        | EffectValue::InitialSpeed(v)
        | EffectValue::InitialStamina(v)
        | EffectValue::InitialPower(v)
        | EffectValue::InitialGuts(v)
        | EffectValue::InitialWit(v)
        | EffectValue::InitialFriendshipGauge(v)
        | EffectValue::RaceBonus(v)
        | EffectValue::FanBonus(v)
        | EffectValue::HintLevels(v)
        | EffectValue::HintFrequency(v)
        | EffectValue::SpecialtyPriority(v)
        | EffectValue::MaxSpeed(v)
        | EffectValue::MaxStamina(v)
        | EffectValue::MaxPower(v)
        | EffectValue::MaxGuts(v)
        | EffectValue::MaxWit(v)
        | EffectValue::EventRecovery(v)
        | EffectValue::EventEffectiveness(v)
        | EffectValue::FailureProtection(v)
        | EffectValue::EnergyCostReduction(v)
        | EffectValue::MinigameEffectiveness(v)
        | EffectValue::SkillPointBonus(v)
        | EffectValue::WitFriendshipRecovery(v)
        | EffectValue::InitialSkillPoints(v)
        | EffectValue::HintQuantityBonus(v)
        | EffectValue::AllStatsBonus(v) => v,
    }
}

pub async fn upsert_all_support_cards(
    pool: &PgPool,
    cards: &[SupportCard],
) -> Result<(), sqlx::Error> {
    if cards.is_empty() {
        return Ok(());
    }

    let support_ids: Vec<i32> = cards.iter().map(|c| c.id.0 as i32).collect();
    let char_names: Vec<&str> = cards.iter().map(|c| c.char_name.as_str()).collect();
    let titles: Vec<&str> = cards.iter().map(|c| c.title.as_str()).collect();
    let card_types: Vec<DbCardType> = cards
        .iter()
        .map(|c| DbCardType::from(c.card_type.clone()))
        .collect();
    let rarities: Vec<DbCardRarity> = cards
        .iter()
        .map(|c| DbCardRarity::from(c.rarity.clone()))
        .collect();
    let is_welfares: Vec<bool> = cards.iter().map(|c| c.is_welfare).collect();
    let release_dates: Vec<Option<chrono::NaiveDate>> =
        cards.iter().map(|c| c.release_date).collect();
    let is_predicted_dates: Vec<bool> = cards.iter().map(|c| c.is_predicted_date).collect();
    let unique_effects: Vec<Option<&str>> =
        cards.iter().map(|c| c.unique_effect.as_deref()).collect();

    sqlx::query!(
        r#"
        INSERT INTO support_cards
            (support_id, char_name, title, card_type, rarity, is_welfare,
             release_en, is_predicted_date, unique_effect)
        SELECT * FROM UNNEST(
            $1::int[], $2::text[], $3::text[], $4::support_card_type[], $5::support_card_rarity[], $6::bool[],
            $7::date[], $8::bool[], $9::text[]
        )
        ON CONFLICT (support_id) DO UPDATE SET
            char_name         = EXCLUDED.char_name,
            title             = EXCLUDED.title,
            card_type         = EXCLUDED.card_type,
            rarity            = EXCLUDED.rarity,
            is_welfare        = EXCLUDED.is_welfare,
            release_en        = EXCLUDED.release_en,
            is_predicted_date = EXCLUDED.is_predicted_date,
            unique_effect     = EXCLUDED.unique_effect
        "#,
        &support_ids,
        &char_names as &[&str],
        &titles as &[&str],
        &card_types as &[DbCardType],
        &rarities as &[DbCardRarity],
        &is_welfares,
        &release_dates as &[Option<chrono::NaiveDate>],
        &is_predicted_dates,
        &unique_effects as &[Option<&str>],
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "DELETE FROM support_card_effects WHERE support_id = ANY($1::int[])",
        &support_ids
    )
    .execute(pool)
    .await?;

    let mut effect_support_ids: Vec<i32> = Vec::new();
    let mut effect_ids: Vec<i32> = Vec::new();
    let mut lb0s: Vec<Option<i32>> = Vec::new();
    let mut lb1s: Vec<Option<i32>> = Vec::new();
    let mut lb2s: Vec<Option<i32>> = Vec::new();
    let mut lb3s: Vec<Option<i32>> = Vec::new();
    let mut mlbs: Vec<Option<i32>> = Vec::new();

    for card in cards {
        for effect in &card.effects {
            let lbs = effect_value_to_lb_values(effect);
            effect_support_ids.push(card.id.0 as i32);
            effect_ids.push(effect_value_to_id(effect));
            lb0s.push(lbs.lb0);
            lb1s.push(lbs.lb1);
            lb2s.push(lbs.lb2);
            lb3s.push(lbs.lb3);
            mlbs.push(lbs.mlb);
        }
    }

    if !effect_support_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO support_card_effects
                (support_id, effect_id, lb0, lb1, lb2, lb3, mlb)
            SELECT * FROM UNNEST(
                $1::int[], $2::int[], $3::int[], $4::int[], $5::int[], $6::int[], $7::int[]
            )
            "#,
            &effect_support_ids,
            &effect_ids,
            &lb0s as &[Option<i32>],
            &lb1s as &[Option<i32>],
            &lb2s as &[Option<i32>],
            &lb3s as &[Option<i32>],
            &mlbs as &[Option<i32>],
        )
        .execute(pool)
        .await?;
    }

    sqlx::query!(
        "DELETE FROM support_card_skills WHERE support_id = ANY($1::int[])",
        &support_ids
    )
    .execute(pool)
    .await?;

    let mut skill_support_ids: Vec<i32> = Vec::new();
    let mut skill_ids: Vec<i32> = Vec::new();
    let mut acquisitions: Vec<String> = Vec::new();

    for card in cards {
        for skill in &card.skills {
            skill_support_ids.push(card.id.0 as i32);
            skill_ids.push(skill.skill_id.0 as i32);
            acquisitions.push(skill.acquisition.to_string());
        }
    }

    if !skill_support_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO support_card_skills (support_id, skill_id, acquisition)
            SELECT * FROM UNNEST($1::int[], $2::int[], $3::support_skill_acquisition[])
            "#,
            &skill_support_ids,
            &skill_ids,
            &acquisitions as &[String],
        )
        .execute(pool)
        .await?;
    }

    log::info!(
        "Support card upsert complete: {} cards, {} effects, {} skills",
        cards.len(),
        effect_support_ids.len(),
        skill_support_ids.len(),
    );

    Ok(())
}

pub async fn get_card_index(pool: &PgPool) -> Result<Vec<SupportCardRow>, sqlx::Error> {
    sqlx::query_as!(
        SupportCardRow,
        r#"
        SELECT support_id, char_name, title,
            card_type as "card_type: DbCardType",
            rarity as "rarity: DbCardRarity",
            is_welfare, release_en, is_predicted_date, unique_effect
        FROM support_cards
        ORDER BY char_name, title
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_card_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<
    Option<(
        SupportCardRow,
        Vec<SupportCardEffectRow>,
        Vec<SupportCardSkillRow>,
    )>,
    sqlx::Error,
> {
    let card = sqlx::query_as!(
        SupportCardRow,
        r#"
        SELECT support_id, char_name, title,
            card_type as "card_type: DbCardType",
            rarity as "rarity: DbCardRarity",
            is_welfare, release_en, is_predicted_date, unique_effect
        FROM support_cards WHERE support_id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(card) = card else {
        return Ok(None);
    };

    let effects = sqlx::query_as!(
        SupportCardEffectRow,
        "SELECT support_id, effect_id, lb0, lb1, lb2, lb3, mlb
         FROM support_card_effects WHERE support_id = $1",
        id
    )
    .fetch_all(pool)
    .await?;

    let skills = sqlx::query_as!(
        SupportCardSkillRow,
        r#"
        SELECT support_id, skill_id,
            acquisition as "acquisition: DbSupportSkillAcquisition"
        FROM support_card_skills WHERE support_id = $1
        "#,
        id
    )
    .fetch_all(pool)
    .await?;

    Ok(Some((card, effects, skills)))
}
