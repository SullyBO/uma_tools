use crate::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uma_db::skill_repo::{get_skill_by_id, get_skills};
use uma_db::types::{DbSkillCategory, DbSkillRarity, SkillFilter};

#[derive(Deserialize)]
pub struct SkillQueryParams {
    pub category: Option<DbSkillCategory>,
    pub rarity: Option<DbSkillRarity>,
    pub is_jp_only: Option<bool>,
    pub effect_type: Option<String>,
}

#[derive(Serialize)]
pub struct SkillSummaryResponse {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub is_jp_only: bool,
}

#[derive(Serialize)]
pub struct SkillDetailResponse {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub is_jp_only: bool,
    pub triggers: Vec<TriggerResponse>,
}

#[derive(Serialize)]
pub struct TriggerResponse {
    pub id: i32,
    pub effects: Vec<EffectResponse>,
    pub conditions: Vec<ConditionResponse>,
    pub preconditions: Vec<ConditionResponse>,
}

#[derive(Serialize)]
pub struct EffectResponse {
    pub effect_type: String,
    pub effect_value: Option<f32>,
}

#[derive(Serialize)]
pub struct ConditionResponse {
    pub cond_key: String,
    pub operator: String,
    pub cond_val: String,
    pub is_or: bool,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<SkillQueryParams>,
) -> Result<Json<Vec<SkillSummaryResponse>>, StatusCode> {
    let filter = SkillFilter {
        category: params.category,
        rarity: params.rarity,
        is_jp_only: params.is_jp_only,
        effect_type: params.effect_type,
    };

    let rows = get_skills(&state.pool, filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let skills = rows
        .into_iter()
        .map(|s| SkillSummaryResponse {
            id: s.id,
            name: s.name,
            category: format!("{:?}", s.category),
            rarity: format!("{:?}", s.rarity),
            sp_cost: s.sp_cost,
            is_jp_only: s.is_jp_only,
        })
        .collect();

    Ok(Json(skills))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SkillDetailResponse>, StatusCode> {
    let detail = get_skill_by_id(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SkillDetailResponse {
        id: detail.skill.id,
        name: detail.skill.name,
        category: format!("{:?}", detail.skill.category),
        rarity: format!("{:?}", detail.skill.rarity),
        sp_cost: detail.skill.sp_cost,
        is_jp_only: detail.skill.is_jp_only,
        triggers: detail
            .triggers
            .into_iter()
            .map(|t| TriggerResponse {
                id: t.id,
                effects: t
                    .effects
                    .into_iter()
                    .map(|e| EffectResponse {
                        effect_type: e.effect_type,
                        effect_value: e.effect_value,
                    })
                    .collect(),
                conditions: t
                    .conditions
                    .into_iter()
                    .map(|c| ConditionResponse {
                        cond_key: c.cond_key,
                        operator: format!("{:?}", c.operator),
                        cond_val: c.cond_val,
                        is_or: c.is_or,
                    })
                    .collect(),
                preconditions: t
                    .preconditions
                    .into_iter()
                    .map(|c| ConditionResponse {
                        cond_key: c.cond_key,
                        operator: format!("{:?}", c.operator),
                        cond_val: c.cond_val,
                        is_or: c.is_or,
                    })
                    .collect(),
            })
            .collect(),
    }))
}
