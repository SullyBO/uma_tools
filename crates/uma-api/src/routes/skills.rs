use crate::AppState;
use crate::error::ApiError;
use crate::routes::models::{
    InheritedSkill, SkillAcquisitionEntry, SkillCondition, SkillDetail, SkillEffect, SkillIndex,
    SkillQueryParams, SkillSummary, SkillTrigger,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use uma_db::repositories::skill_repo::get_skill_acquisitions;
use uma_db::{
    repositories::skill_repo::{get_skill_by_id, get_skill_index, get_skills},
    types::{SkillFilter, TriggerRow},
};

pub async fn index(State(state): State<AppState>) -> Result<Json<Vec<SkillIndex>>, ApiError> {
    let rows = get_skill_index(&state.pool).await?;
    let index = rows
        .into_iter()
        .map(|r| SkillIndex { id: r.id, name: r.name })
        .collect();
    Ok(Json(index))
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
            ingame_description: s.ingame_description,
            category: s.category.to_string(),
            rarity: s.rarity.to_string(),
            sp_cost: s.sp_cost,
            is_jp_only: s.is_jp_only,
        })
        .collect();

    Ok(Json(skills))
}

fn map_triggers(triggers: Vec<TriggerRow>) -> Vec<SkillTrigger> {
    triggers
        .into_iter()
        .map(|t| SkillTrigger {
            id: t.id,
            duration: t.duration,
            scaling: t.scaling,
            effects: t
                .effects
                .into_iter()
                .map(|e| SkillEffect {
                    effect_type: e.effect_type,
                    effect_value: e.effect_value,
                })
                .collect(),
            conditions: t
                .conditions
                .into_iter()
                .map(|c| SkillCondition {
                    cond_key: c.cond_key,
                    operator: c.operator,
                    cond_val: c.cond_val,
                    is_or: c.is_or,
                })
                .collect(),
            preconditions: t
                .preconditions
                .into_iter()
                .map(|c| SkillCondition {
                    cond_key: c.cond_key,
                    operator: c.operator,
                    cond_val: c.cond_val,
                    is_or: c.is_or,
                })
                .collect(),
        })
        .collect()
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SkillDetail>, ApiError> {
    let (skill, triggers, inherited) = get_skill_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let acquisitions = get_skill_acquisitions(&state.pool, id).await?;

    let inherited_skill = inherited.map(|(s, t)| InheritedSkill {
        id: s.id,
        name: s.name,
        ingame_description: s.ingame_description,
        category: s.category.to_string(),
        rarity: s.rarity.to_string(),
        sp_cost: s.sp_cost,
        triggers: map_triggers(t),
    });

    Ok(Json(SkillDetail {
        id: skill.id,
        name: skill.name,
        ingame_description: skill.ingame_description,
        category: skill.category.to_string(),
        rarity: skill.rarity.to_string(),
        sp_cost: skill.sp_cost,
        is_jp_only: skill.is_jp_only,
        inherited_skill,
        triggers: map_triggers(triggers),
        acquisitions: acquisitions
            .into_iter()
            .map(|a| SkillAcquisitionEntry {
                source_id: a.source_id,
                source_type: a.source_type,
                acquisition: a.acquisition,
            })
            .collect(),
    }))
}
