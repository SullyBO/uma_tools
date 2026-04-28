use sqlx::Type;
use uma_core::{
    models::{
        skill::{Category, Operator, Rarity as SkillRarity},
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

#[derive(Debug, Type)]
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

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_category")]
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

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_rarity")]
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

#[derive(Debug, Type)]
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
