use crate::AppState;
use crate::error::ApiError;
use crate::routes::models::{
    SkillCondition, SkillDetail, SkillEffect, SkillIndex, SkillQueryParams, SkillSummary, SkillTrigger
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use uma_db::skill_repo::{get_skill_by_id, get_skills};
use uma_db::types::SkillFilter;

pub async fn index(State(state): State<AppState>) -> Result<Json<Vec<SkillIndex>>, ApiError> {
    let rows = sqlx::query_as!(
        SkillIndex,
        "SELECT id, name FROM skills ORDER BY name"
    )
    .fetch_all(&*state.pool)
    .await?;

    Ok(Json(rows))
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<SkillQueryParams>,
) -> Result<Json<Vec<SkillSummary>>, ApiError> {
    let filter = SkillFilter {
        category: params.category,
        rarity: params.rarity,
        is_jp_only: params.is_jp_only,
        effect_type: params.effect_type,
    };

    let rows = get_skills(&state.pool, filter).await?;

    let skills = rows
        .into_iter()
        .map(|s| SkillSummary {
            id: s.id,
            name: s.name,
            category: s.category.to_string(),
            rarity: s.rarity.to_string(),
            sp_cost: s.sp_cost,
            is_jp_only: s.is_jp_only,
        })
        .collect();

    Ok(Json(skills))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SkillDetail>, ApiError> {
    let map_condition = |c: uma_db::types::ConditionRow| SkillCondition {
        cond_key: c.cond_key,
        operator: c.operator.to_string(),
        cond_val: c.cond_val,
        is_or: c.is_or,
    };

    let detail = get_skill_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(SkillDetail {
        id: detail.skill.id,
        name: detail.skill.name,
        category: detail.skill.category.to_string(),
        rarity: detail.skill.rarity.to_string(),
        sp_cost: detail.skill.sp_cost,
        is_jp_only: detail.skill.is_jp_only,
        triggers: detail
            .triggers
            .into_iter()
            .map(|t| SkillTrigger {
                id: t.id,
                effects: t
                    .effects
                    .into_iter()
                    .map(|e| SkillEffect {
                        effect_type: e.effect_type,
                        effect_value: e.effect_value,
                    })
                    .collect(),
                conditions: t.conditions.into_iter().map(map_condition).collect(),
                preconditions: t.preconditions.into_iter().map(map_condition).collect(),
            })
            .collect(),
    }))
}
