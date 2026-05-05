use crate::AppState;
use crate::error::ApiError;
use crate::routes::models::{UmaDetail, UmaIndex, UmaQueryParams, UmaSkillEntry, UmaSummary};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use uma_db::{
    uma_skill_repo::{get_skills_for_uma, get_uma_by_id},
    {types::UmaFilter, uma_repo::get_umas},
};

pub async fn index(State(state): State<AppState>) -> Result<Json<Vec<UmaIndex>>, ApiError> {
    let rows = sqlx::query_as!(
        UmaIndex,
        "SELECT id, name, subtitle AS version FROM umas ORDER BY name, subtitle"
    )
    .fetch_all(&*state.pool)
    .await?;

    Ok(Json(rows))
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<UmaQueryParams>,
) -> Result<Json<Vec<UmaSummary>>, ApiError> {
    let filter = UmaFilter {
        turf: params.turf,
        dirt: params.dirt,
        short: params.short,
        mile: params.mile,
        medium: params.medium,
        long: params.long,
        front: params.front,
        pace: params.pace,
        late: params.late,
        end: params.end,
    };

    let rows = get_umas(&state.pool, filter).await?;

    let umas = rows
        .into_iter()
        .map(|r| UmaSummary {
            id: r.id,
            name: r.name,
            subtitle: r.subtitle,
            apt_turf: format!("{:?}", r.apt_turf),
            apt_dirt: format!("{:?}", r.apt_dirt),
            apt_short: format!("{:?}", r.apt_short),
            apt_mile: format!("{:?}", r.apt_mile),
            apt_medium: format!("{:?}", r.apt_medium),
            apt_long: format!("{:?}", r.apt_long),
            apt_front: format!("{:?}", r.apt_front),
            apt_pace: format!("{:?}", r.apt_pace),
            apt_late: format!("{:?}", r.apt_late),
            apt_end: format!("{:?}", r.apt_end),
        })
        .collect();

    Ok(Json(umas))
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UmaDetail>, ApiError> {
    let uma = get_uma_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let skills = get_skills_for_uma(&state.pool, id).await?;

    Ok(Json(UmaDetail {
        id: uma.id,
        name: uma.name,
        subtitle: uma.subtitle,
        apt_turf: format!("{:?}", uma.apt_turf),
        apt_dirt: format!("{:?}", uma.apt_dirt),
        apt_short: format!("{:?}", uma.apt_short),
        apt_mile: format!("{:?}", uma.apt_mile),
        apt_medium: format!("{:?}", uma.apt_medium),
        apt_long: format!("{:?}", uma.apt_long),
        apt_front: format!("{:?}", uma.apt_front),
        apt_pace: format!("{:?}", uma.apt_pace),
        apt_late: format!("{:?}", uma.apt_late),
        apt_end: format!("{:?}", uma.apt_end),
        skills: skills
            .into_iter()
            .map(|s| UmaSkillEntry {
                id: s.id,
                name: s.name,
                category: format!("{:?}", s.category),
                rarity: format!("{:?}", s.rarity),
                sp_cost: s.sp_cost,
                acquisition: format!("{:?}", s.acquisition),
                evolved_from: s.evolved_from,
            })
            .collect(),
    }))
}
