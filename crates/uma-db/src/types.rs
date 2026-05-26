use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::Type;
use std::fmt;
use uma_core::{
    models::{
        skill::{Category, Operator, Rarity as SkillRarity},
        support_card::{CardType, Rarity as SupportRarity},
        uma::{AptitudeLevel, Rarity as UmaRarity},
    },
    uma_skill::SkillAcquisition,
};

#[derive(Debug, Type)]
#[sqlx(type_name = "uma_rarity")]
pub enum DbUmaRarity {
    #[sqlx(rename = "r")]
    R,
    #[sqlx(rename = "sr")]
    SR,
    #[sqlx(rename = "ssr")]
    SSR,
}

impl fmt::Display for DbUmaRarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbUmaRarity::R => write!(f, "r"),
            DbUmaRarity::SR => write!(f, "sr"),
            DbUmaRarity::SSR => write!(f, "ssr"),
        }
    }
}

#[derive(Debug, Type, Deserialize)]
#[sqlx(type_name = "aptitude_level")]
pub enum DbAptitudeLevel {
    #[sqlx(rename = "a")]
    A,
    #[sqlx(rename = "b")]
    B,
    #[sqlx(rename = "c")]
    C,
    #[sqlx(rename = "d")]
    D,
    #[sqlx(rename = "e")]
    E,
    #[sqlx(rename = "f")]
    F,
    #[sqlx(rename = "g")]
    G,
}

impl fmt::Display for DbAptitudeLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbAptitudeLevel::A => write!(f, "A"),
            DbAptitudeLevel::B => write!(f, "B"),
            DbAptitudeLevel::C => write!(f, "C"),
            DbAptitudeLevel::D => write!(f, "D"),
            DbAptitudeLevel::E => write!(f, "E"),
            DbAptitudeLevel::F => write!(f, "F"),
            DbAptitudeLevel::G => write!(f, "G"),
        }
    }
}

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_acquisition", rename_all = "PascalCase")]
pub enum DbSkillAcquisition {
    #[sqlx(rename = "unique")]
    Unique,
    #[sqlx(rename = "innate")]
    Innate,
    #[sqlx(rename = "awakening")]
    Awakening,
    #[sqlx(rename = "event")]
    Event,
    #[sqlx(rename = "evolution")]
    Evolution,
}

impl fmt::Display for DbSkillAcquisition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbSkillAcquisition::Unique => write!(f, "unique"),
            DbSkillAcquisition::Innate => write!(f, "innate"),
            DbSkillAcquisition::Awakening => write!(f, "awakening"),
            DbSkillAcquisition::Event => write!(f, "event"),
            DbSkillAcquisition::Evolution => write!(f, "evolution"),
        }
    }
}

impl From<UmaRarity> for DbUmaRarity {
    fn from(r: UmaRarity) -> Self {
        match r {
            UmaRarity::R => DbUmaRarity::R,
            UmaRarity::SR => DbUmaRarity::SR,
            UmaRarity::SSR => DbUmaRarity::SSR,
        }
    }
}

impl From<AptitudeLevel> for DbAptitudeLevel {
    fn from(a: AptitudeLevel) -> Self {
        match a {
            AptitudeLevel::A => DbAptitudeLevel::A,
            AptitudeLevel::B => DbAptitudeLevel::B,
            AptitudeLevel::C => DbAptitudeLevel::C,
            AptitudeLevel::D => DbAptitudeLevel::D,
            AptitudeLevel::E => DbAptitudeLevel::E,
            AptitudeLevel::F => DbAptitudeLevel::F,
            AptitudeLevel::G => DbAptitudeLevel::G,
        }
    }
}

impl From<SkillAcquisition> for DbSkillAcquisition {
    fn from(a: SkillAcquisition) -> Self {
        match a {
            SkillAcquisition::Unique => DbSkillAcquisition::Unique,
            SkillAcquisition::Innate => DbSkillAcquisition::Innate,
            SkillAcquisition::Awakening => DbSkillAcquisition::Awakening,
            SkillAcquisition::Event => DbSkillAcquisition::Event,
            SkillAcquisition::Evolution(_) => DbSkillAcquisition::Evolution,
        }
    }
}

#[derive(Debug, Type, Deserialize)]
#[sqlx(type_name = "skill_category")]
#[serde(rename_all = "snake_case")]
pub enum DbSkillCategory {
    #[sqlx(rename = "green")]
    Green,
    #[sqlx(rename = "recovery")]
    Recovery,
    #[sqlx(rename = "velocity")]
    Velocity,
    #[sqlx(rename = "acceleration")]
    Acceleration,
    #[sqlx(rename = "movement")]
    Movement,
    #[sqlx(rename = "gate")]
    Gate,
    #[sqlx(rename = "vision")]
    Vision,
    #[sqlx(rename = "speed_debuff")]
    SpeedDebuff,
    #[sqlx(rename = "accel_debuff")]
    AccelDebuff,
    #[sqlx(rename = "frenzy_debuff")]
    FrenzyDebuff,
    #[sqlx(rename = "stamina_drain")]
    StaminaDrain,
    #[sqlx(rename = "vision_debuff")]
    VisionDebuff,
    #[sqlx(rename = "purple")]
    Purple,
    #[sqlx(rename = "scenario")]
    Scenario,
    #[sqlx(rename = "unique")]
    Unique,
    #[sqlx(rename = "unique_recovery")]
    UniqueRecovery,
    #[sqlx(rename = "zenkai")]
    Zenkai,
}

impl fmt::Display for DbSkillCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbSkillCategory::Green => write!(f, "green"),
            DbSkillCategory::Recovery => write!(f, "recovery"),
            DbSkillCategory::Velocity => write!(f, "velocity"),
            DbSkillCategory::Acceleration => write!(f, "acceleration"),
            DbSkillCategory::Movement => write!(f, "movement"),
            DbSkillCategory::Gate => write!(f, "gate"),
            DbSkillCategory::Vision => write!(f, "vision"),
            DbSkillCategory::SpeedDebuff => write!(f, "speed_debuff"),
            DbSkillCategory::AccelDebuff => write!(f, "accel_debuff"),
            DbSkillCategory::FrenzyDebuff => write!(f, "frenzy_debuff"),
            DbSkillCategory::StaminaDrain => write!(f, "stamina_drain"),
            DbSkillCategory::VisionDebuff => write!(f, "vision_debuff"),
            DbSkillCategory::Purple => write!(f, "purple"),
            DbSkillCategory::Scenario => write!(f, "scenario"),
            DbSkillCategory::Unique => write!(f, "unique"),
            DbSkillCategory::UniqueRecovery => write!(f, "unique_recovery"),
            DbSkillCategory::Zenkai => write!(f, "zenkai"),
        }
    }
}

#[derive(Debug, Type, Deserialize)]
#[sqlx(type_name = "skill_rarity")]
#[serde(rename_all = "snake_case")]
pub enum DbSkillRarity {
    #[sqlx(rename = "normal")]
    Normal,
    #[sqlx(rename = "rare")]
    Rare,
    #[sqlx(rename = "unique")]
    Unique,
    #[sqlx(rename = "evolution")]
    Evolution,
}

impl fmt::Display for DbSkillRarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbSkillRarity::Normal => write!(f, "normal"),
            DbSkillRarity::Rare => write!(f, "rare"),
            DbSkillRarity::Unique => write!(f, "unique"),
            DbSkillRarity::Evolution => write!(f, "evolution"),
        }
    }
}

impl From<Category> for DbSkillCategory {
    fn from(c: Category) -> Self {
        match c {
            Category::Green => DbSkillCategory::Green,
            Category::Recovery => DbSkillCategory::Recovery,
            Category::Velocity => DbSkillCategory::Velocity,
            Category::Acceleration => DbSkillCategory::Acceleration,
            Category::Movement => DbSkillCategory::Movement,
            Category::Gate => DbSkillCategory::Gate,
            Category::Vision => DbSkillCategory::Vision,
            Category::SpeedDebuff => DbSkillCategory::SpeedDebuff,
            Category::AccelDebuff => DbSkillCategory::AccelDebuff,
            Category::FrenzyDebuff => DbSkillCategory::FrenzyDebuff,
            Category::StaminaDrain => DbSkillCategory::StaminaDrain,
            Category::VisionDebuff => DbSkillCategory::VisionDebuff,
            Category::Purple => DbSkillCategory::Purple,
            Category::Scenario => DbSkillCategory::Scenario,
            Category::Unique => DbSkillCategory::Unique,
            Category::UniqueRecovery => DbSkillCategory::UniqueRecovery,
            Category::Zenkai => DbSkillCategory::Zenkai,
        }
    }
}

impl From<SkillRarity> for DbSkillRarity {
    fn from(r: SkillRarity) -> Self {
        match r {
            SkillRarity::Normal => DbSkillRarity::Normal,
            SkillRarity::Rare => DbSkillRarity::Rare,
            SkillRarity::Unique => DbSkillRarity::Unique,
            SkillRarity::Evolution => DbSkillRarity::Evolution,
        }
    }
}

#[derive(Debug, Clone, Copy, Type, Deserialize)]
#[sqlx(type_name = "skill_operator")]
pub enum DbSkillOperator {
    #[sqlx(rename = "eq")]
    Eq,
    #[sqlx(rename = "not_eq")]
    NotEq,
    #[sqlx(rename = "gt")]
    Gt,
    #[sqlx(rename = "gt_eq")]
    GtEq,
    #[sqlx(rename = "lt")]
    Lt,
    #[sqlx(rename = "lt_eq")]
    LtEq,
}

impl fmt::Display for DbSkillOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbSkillOperator::Eq => write!(f, "eq"),
            DbSkillOperator::NotEq => write!(f, "not_eq"),
            DbSkillOperator::Gt => write!(f, "gt"),
            DbSkillOperator::GtEq => write!(f, "gt_eq"),
            DbSkillOperator::Lt => write!(f, "lt"),
            DbSkillOperator::LtEq => write!(f, "lt_eq"),
        }
    }
}

impl From<Operator> for DbSkillOperator {
    fn from(o: Operator) -> Self {
        match o {
            Operator::Eq => DbSkillOperator::Eq,
            Operator::NotEq => DbSkillOperator::NotEq,
            Operator::Gt => DbSkillOperator::Gt,
            Operator::GtEq => DbSkillOperator::GtEq,
            Operator::Lt => DbSkillOperator::Lt,
            Operator::LtEq => DbSkillOperator::LtEq,
        }
    }
}

pub struct UmaFilter {
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

#[derive(sqlx::FromRow)]
pub struct UmaRow {
    pub id: i32,
    pub name: String,
    pub subtitle: String,
    pub apt_turf: DbAptitudeLevel,
    pub apt_dirt: DbAptitudeLevel,
    pub apt_short: DbAptitudeLevel,
    pub apt_mile: DbAptitudeLevel,
    pub apt_medium: DbAptitudeLevel,
    pub apt_long: DbAptitudeLevel,
    pub apt_front: DbAptitudeLevel,
    pub apt_pace: DbAptitudeLevel,
    pub apt_late: DbAptitudeLevel,
    pub apt_end: DbAptitudeLevel,
    pub growth_speed: i32,
    pub growth_stamina: i32,
    pub growth_power: i32,
    pub growth_guts: i32,
    pub growth_wit: i32,
    pub release_date: NaiveDate,
    pub is_predicted_date: bool,
}

#[derive(sqlx::FromRow)]
pub struct UmaSummaryRow {
    pub id: i32,
    pub name: String,
    pub subtitle: String,
    pub apt_turf: DbAptitudeLevel,
    pub apt_dirt: DbAptitudeLevel,
    pub apt_short: DbAptitudeLevel,
    pub apt_mile: DbAptitudeLevel,
    pub apt_medium: DbAptitudeLevel,
    pub apt_long: DbAptitudeLevel,
    pub apt_front: DbAptitudeLevel,
    pub apt_pace: DbAptitudeLevel,
    pub apt_late: DbAptitudeLevel,
    pub apt_end: DbAptitudeLevel,
    pub release_date: NaiveDate,
    pub is_predicted_date: bool,
}

#[derive(sqlx::FromRow)]
pub struct UmaSkillRow {
    pub id: i32,
    pub name: String,
    pub category: DbSkillCategory,
    pub rarity: DbSkillRarity,
    pub sp_cost: i32,
    pub acquisition: DbSkillAcquisition,
    pub evolved_from: Option<i32>,
}

#[derive(sqlx::FromRow)]
pub struct SkillRow {
    pub id: i32,
    pub name: String,
    pub ingame_description: String,
    pub category: DbSkillCategory,
    pub rarity: DbSkillRarity,
    pub sp_cost: i32,
    pub is_jp_only: bool,
}

pub struct SkillFilter {
    pub category: Option<DbSkillCategory>,
    pub rarity: Option<DbSkillRarity>,
    pub is_jp_only: Option<bool>,
    pub effect_type: Option<String>,
}

pub struct SkillDetail {
    pub skill: SkillRow,
    pub triggers: Vec<TriggerRow>,
}

pub struct TriggerRow {
    pub id: i32,
    pub duration: Option<f32>,
    pub scaling: Option<String>,
    pub effects: Vec<EffectRow>,
    pub conditions: Vec<ConditionRow>,
    pub preconditions: Vec<ConditionRow>,
}

pub struct EffectRow {
    pub trigger_id: i32,
    pub effect_type: String,
    pub effect_value: Option<f32>,
}

pub struct ConditionRow {
    pub cond_key: String,
    pub operator: DbSkillOperator,
    pub cond_val: String,
    pub is_or: bool,
}

pub struct SupportCardFilter {
    pub card_type: Option<DbCardType>,
    pub rarity: Option<DbCardRarity>,
    pub is_welfare: Option<bool>,
}

#[derive(sqlx::FromRow)]
pub struct SupportCardRow {
    pub support_id: i32,
    pub char_name: String,
    pub title: String,
    pub card_type: DbCardType,
    pub rarity: DbCardRarity,
    pub is_welfare: bool,
    pub release_en: Option<NaiveDate>,
    pub is_predicted_date: bool,
    pub unique_effect: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct SupportCardEffectRow {
    pub support_id: i32,
    pub effect_id: i32,
    pub lb0: Option<i32>,
    pub lb1: Option<i32>,
    pub lb2: Option<i32>,
    pub lb3: Option<i32>,
    pub mlb: Option<i32>,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "support_card_type")]
pub enum DbCardType {
    #[sqlx(rename = "speed")]
    Speed,
    #[sqlx(rename = "stamina")]
    Stamina,
    #[sqlx(rename = "power")]
    Power,
    #[sqlx(rename = "guts")]
    Guts,
    #[sqlx(rename = "wit")]
    Wit,
    #[sqlx(rename = "friend")]
    Friend,
    #[sqlx(rename = "group")]
    Group,
}

impl From<CardType> for DbCardType {
    fn from(ct: CardType) -> Self {
        match ct {
            CardType::Speed => DbCardType::Speed,
            CardType::Stamina => DbCardType::Stamina,
            CardType::Power => DbCardType::Power,
            CardType::Guts => DbCardType::Guts,
            CardType::Wit => DbCardType::Wit,
            CardType::Friend => DbCardType::Friend,
            CardType::Group => DbCardType::Group,
        }
    }
}

impl fmt::Display for DbCardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbCardType::Speed => write!(f, "speed"),
            DbCardType::Stamina => write!(f, "stamina"),
            DbCardType::Power => write!(f, "power"),
            DbCardType::Guts => write!(f, "guts"),
            DbCardType::Wit => write!(f, "wit"),
            DbCardType::Friend => write!(f, "friend"),
            DbCardType::Group => write!(f, "group"),
        }
    }
}

#[derive(Debug, Type, Clone)]
#[sqlx(type_name = "support_card_rarity")]
pub enum DbCardRarity {
    #[sqlx(rename = "r")]
    R,
    #[sqlx(rename = "sr")]
    SR,
    #[sqlx(rename = "ssr")]
    SSR,
}

impl fmt::Display for DbCardRarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbCardRarity::R => write!(f, "r"),
            DbCardRarity::SR => write!(f, "sr"),
            DbCardRarity::SSR => write!(f, "ssr"),
        }
    }
}

impl From<SupportRarity> for DbCardRarity {
    fn from(r: SupportRarity) -> Self {
        match r {
            SupportRarity::R => DbCardRarity::R,
            SupportRarity::SR => DbCardRarity::SR,
            SupportRarity::SSR => DbCardRarity::SSR,
        }
    }
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(type_name = "support_skill_acquisition")]
pub enum DbSupportSkillAcquisition {
    #[sqlx(rename = "event")]
    Event,
    #[sqlx(rename = "hint")]
    Hint,
}

impl fmt::Display for DbSupportSkillAcquisition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbSupportSkillAcquisition::Event => write!(f, "event"),
            DbSupportSkillAcquisition::Hint => write!(f, "hint"),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct SupportCardSkillRow {
    pub support_id: i32,
    pub skill_id: i32,
    pub acquisition: DbSupportSkillAcquisition,
}

pub struct AcquisitionRow {
    pub source_id: i32,
    pub source_type: String,
    pub acquisition: String,
}
