use crate::AppState;
use crate::error::ApiError;
use crate::routes::models::{CardDetail, CardEffect, CardIndex, CardSkill};
use axum::{
    Json,
    extract::{Path, State},
};
use uma_db::repositories::support_card_repo::{get_card_by_id, get_card_index};

fn effect_id_to_name(id: i32) -> &'static str {
    match id {
        1 => "Friendship Bonus",
        2 => "Mood Effect",
        3 => "Speed Bonus",
        4 => "Stamina Bonus",
        5 => "Power Bonus",
        6 => "Guts Bonus",
        7 => "Wit Bonus",
        8 => "Training Effectiveness",
        9 => "Initial Speed",
        10 => "Initial Stamina",
        11 => "Initial Power",
        12 => "Initial Guts",
        13 => "Initial Wit",
        14 => "Initial Friendship Gauge",
        15 => "Race Bonus",
        16 => "Fan Bonus",
        17 => "Hint Levels",
        18 => "Hint Frequency",
        19 => "Specialty Priority",
        20 => "Max Speed",
        21 => "Max Stamina",
        22 => "Max Power",
        23 => "Max Guts",
        24 => "Max Wit",
        25 => "Event Recovery",
        26 => "Event Effectiveness",
        27 => "Failure Protection",
        28 => "Energy Cost Reduction",
        29 => "Minigame Effectiveness",
        30 => "Skill Point Bonus",
        31 => "Wit Friendship Recovery",
        32 => "Initial Skill Points",
        33 => "Hint Quantity Bonus",
        41 => "All Stats Bonus",
        _ => "Unknown",
    }
}

pub async fn index(State(state): State<AppState>) -> Result<Json<Vec<CardIndex>>, ApiError> {
    let rows = get_card_index(&state.pool).await?;
 
    let cards = rows
        .into_iter()
        .map(|r| CardIndex {
            support_id: r.support_id,
            char_name: r.char_name,
            title: r.title,
            card_type: r.card_type.to_string(),
            rarity: r.rarity.to_string(),
            is_welfare: r.is_welfare,
            release_date: r.release_en.map(|d| d.to_string()),
            is_predicted_date: r.is_predicted_date,
        })
        .collect();
 
    Ok(Json(cards))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<CardDetail>, ApiError> {
    let (card, effects, skills) = get_card_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(CardDetail {
        support_id: card.support_id,
        char_name: card.char_name,
        title: card.title,
        card_type: card.card_type.to_string(),
        rarity: card.rarity.to_string(),
        is_welfare: card.is_welfare,
        release_date: card.release_en.map(|d| d.to_string()),
        is_predicted_date: card.is_predicted_date,
        unique_effect: card.unique_effect,
        effects: effects
            .into_iter()
            .map(|e| CardEffect {
                effect_name: effect_id_to_name(e.effect_id).to_string(),
                lb0: e.lb0,
                lb1: e.lb1,
                lb2: e.lb2,
                lb3: e.lb3,
                mlb: e.mlb,
            })
            .collect(),
        skills: skills
            .into_iter()
            .map(|s| CardSkill {
                skill_id: s.skill_id,
                acquisition: s.acquisition.to_string(),
            })
            .collect(),
    }))
}
