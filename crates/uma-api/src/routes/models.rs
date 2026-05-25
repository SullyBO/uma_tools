use serde::{Deserialize, Serialize};
use uma_db::types::{DbAptitudeLevel, DbSkillCategory, DbSkillRarity};

#[derive(Debug, Serialize)]
pub struct UmaIndex {
    pub id: i32,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct SkillIndex {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct UmaQueryParams {
    pub released: Option<bool>,
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
    pub release_date: String,
    pub is_predicted_date: bool,
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
    pub growth_speed: i32,
    pub growth_stamina: i32,
    pub growth_power: i32,
    pub growth_guts: i32,
    pub growth_wit: i32,
    pub release_date: String,
    pub is_predicted_date: bool,
    pub skills: Vec<UmaSkillEntry>,
}

#[derive(Serialize)]
pub struct UmaSkillEntry {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub acquisition: String,
    pub evolved_from: Option<i32>,
}

#[derive(Deserialize)]
pub struct SkillQueryParams {
    pub category: Option<DbSkillCategory>,
    pub rarity: Option<DbSkillRarity>,
    pub is_jp_only: Option<bool>,
    pub effect_type: Option<String>,
}

#[derive(Serialize)]
pub struct SkillSummary {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub is_jp_only: bool,
}

#[derive(Serialize)]
pub struct SkillDetail {
    pub id: i32,
    pub name: String,
    pub ingame_description: String,
    pub category: String,
    pub rarity: String,
    pub sp_cost: i32,
    pub is_jp_only: bool,
    pub triggers: Vec<SkillTrigger>,
}

#[derive(Serialize)]
pub struct SkillTrigger {
    pub id: i32,
    pub duration: Option<f32>,
    pub scaling: Option<String>,
    pub effects: Vec<SkillEffect>,
    pub conditions: Vec<SkillCondition>,
    pub preconditions: Vec<SkillCondition>,
}

#[derive(Serialize)]
pub struct SkillEffect {
    pub effect_type: String,
    pub effect_value: Option<f32>,
}

#[derive(Serialize)]
pub struct SkillCondition {
    pub cond_key: String,
    pub operator: String,
    pub cond_val: String,
    pub is_or: bool,
}

#[derive(Debug, Serialize)]
pub struct CardIndex {
    pub support_id: i32,
    pub char_name: String,
    pub title: String,
    pub card_type: String,
    pub rarity: String,
    pub is_welfare: bool,
    pub release_date: Option<String>,
    pub is_predicted_date: bool,
}

#[derive(Serialize)]
pub struct CardDetail {
    pub support_id: i32,
    pub char_name: String,
    pub title: String,
    pub card_type: String,
    pub rarity: String,
    pub is_welfare: bool,
    pub release_date: Option<String>,
    pub is_predicted_date: bool,
    pub unique_effect: Option<String>,
    pub effects: Vec<CardEffect>,
    pub skills: Vec<CardSkill>,
}

#[derive(Serialize)]
pub struct CardEffect {
    pub effect_name: String,
    pub lb0: Option<i32>,
    pub lb1: Option<i32>,
    pub lb2: Option<i32>,
    pub lb3: Option<i32>,
    pub mlb: Option<i32>,
}

#[derive(Serialize)]
pub struct CardSkill {
    pub skill_id: i32,
    pub acquisition: String,
}
