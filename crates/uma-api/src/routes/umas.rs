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
        released: params.released,
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
            apt_turf: r.apt_turf.to_string(),
            apt_dirt: r.apt_dirt.to_string(),
            apt_short: r.apt_short.to_string(),
            apt_mile: r.apt_mile.to_string(),
            apt_medium: r.apt_medium.to_string(),
            apt_long: r.apt_long.to_string(),
            apt_front: r.apt_front.to_string(),
            apt_pace: r.apt_pace.to_string(),
            apt_late: r.apt_late.to_string(),
            apt_end: r.apt_end.to_string(),
            release_date: r.release_date.to_string(),
            is_predicted_date: r.is_predicted_date,
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
        apt_turf: uma.apt_turf.to_string(),
        apt_dirt: uma.apt_dirt.to_string(),
        apt_short: uma.apt_short.to_string(),
        apt_mile: uma.apt_mile.to_string(),
        apt_medium: uma.apt_medium.to_string(),
        apt_long: uma.apt_long.to_string(),
        apt_front: uma.apt_front.to_string(),
        apt_pace: uma.apt_pace.to_string(),
        apt_late: uma.apt_late.to_string(),
        apt_end: uma.apt_end.to_string(),
        growth_speed: uma.growth_speed,
        growth_stamina: uma.growth_stamina,
        growth_power: uma.growth_power,
        growth_guts: uma.growth_guts,
        growth_wit: uma.growth_wit,
        release_date: uma.release_date.to_string(),
        is_predicted_date: uma.is_predicted_date,
        skills: skills
            .into_iter()
            .map(|s| UmaSkillEntry {
                id: s.id,
                name: s.name,
                category: s.category.to_string(),
                rarity: s.rarity.to_string(),
                sp_cost: s.sp_cost,
                acquisition: s.acquisition.to_string(),
                evolved_from: s.evolved_from,
            })
            .collect(),
    }))
}
