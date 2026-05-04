use crate::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uma_db::types::{DbAptitudeLevel, UmaFilter};
use uma_db::uma_repo::get_umas;
use uma_db::uma_skill_repo::{get_skills_for_uma, get_uma_by_id};

#[derive(Deserialize)]
pub struct UmaQueryParams {
    pub turf: Option<DbAptitudeLevel>,
    pub dirt: Option<DbAptitudeLevel>,
    pub short: Option<DbAptitudeLevel>,
    pub mile: Option<DbAptitudeLevel>,
    pub medium: Option<DbAptitudeLevel>,
    pub long: Option<DbAptitudeLevel>,
    pub front: Option<DbAptitudeLevel>,
    pub pace: Option<DbAptitudeLevel>,
    pub late: Option<DbAptitudeLevel>,
    pub end: Option<DbAptitudeLevel>,
}

#[derive(Serialize)]
pub struct UmaSummary {
    pub id: i32,
    pub name: String,
    pub subtitle: String,
    pub apt_turf: String,
    pub apt_dirt: String,
    pub apt_short: String,
    pub apt_mile: String,
    pub apt_medium: String,
    pub apt_long: String,
    pub apt_front: String,
    pub apt_pace: String,
    pub apt_late: String,
    pub apt_end: String,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<UmaQueryParams>,
) -> Result<Json<Vec<UmaSummary>>, StatusCode> {
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

    let rows = get_umas(&state.pool, filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

#[derive(Serialize)]
pub struct UmaDetail {
    pub id: i32,
    pub name: String,
    pub subtitle: String,
    pub apt_turf: String,
    pub apt_dirt: String,
    pub apt_short: String,
    pub apt_mile: String,
    pub apt_medium: String,
    pub apt_long: String,
    pub apt_front: String,
    pub apt_pace: String,
    pub apt_late: String,
    pub apt_end: String,
    pub skills: Vec<SkillSummary>,
}

#[derive(Serialize)]
pub struct SkillSummary {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub acquisition: String,
    pub evolved_from: Option<i32>,
}

pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UmaDetail>, StatusCode> {
    let uma = get_uma_by_id(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let skills = get_skills_for_uma(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
            .map(|s| SkillSummary {
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
