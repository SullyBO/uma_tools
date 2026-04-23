use sqlx::Type;
use uma_core::{
    models::{
        skill::{SkillCategory, SkillOperator, SkillRarity},
        uma::{AptitudeLevel, UmaRarity},
    },
    uma_skill::SkillAcquisition,
};

#[derive(Debug, Type)]
#[sqlx(type_name = "uma_rarity")]
pub enum DbUmaRarity {
    R,
    SR,
    SSR,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "aptitude_level", rename_all = "PascalCase")]
pub enum DbAptitudeLevel {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_acquisition", rename_all = "PascalCase")]
pub enum DbSkillAcquisition {
    Unique,
    Innate,
    Awakening,
    Event,
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
            SkillAcquisition::Evolution => DbSkillAcquisition::Evolution,
        }
    }
}

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_category")]
pub enum DbSkillCategory {
    Green,
    Recovery,
    Velocity,
    Acceleration,
    Movement,
    Gate,
    Vision,
    SpeedDebuff,
    AccelDebuff,
    FrenzyDebuff,
    StaminaDrain,
    VisionDebuff,
    Purple,
    Scenario,
}

#[derive(Debug, Type)]
#[sqlx(type_name = "skill_rarity")]
pub enum DbSkillRarity {
    Normal,
    Rare,
    Unique,
    Evolution,
}

impl From<SkillCategory> for DbSkillCategory {
    fn from(c: SkillCategory) -> Self {
        match c {
            SkillCategory::Green => DbSkillCategory::Green,
            SkillCategory::Recovery => DbSkillCategory::Recovery,
            SkillCategory::Velocity => DbSkillCategory::Velocity,
            SkillCategory::Acceleration => DbSkillCategory::Acceleration,
            SkillCategory::Movement => DbSkillCategory::Movement,
            SkillCategory::Gate => DbSkillCategory::Gate,
            SkillCategory::Vision => DbSkillCategory::Vision,
            SkillCategory::SpeedDebuff => DbSkillCategory::SpeedDebuff,
            SkillCategory::AccelDebuff => DbSkillCategory::AccelDebuff,
            SkillCategory::FrenzyDebuff => DbSkillCategory::FrenzyDebuff,
            SkillCategory::StaminaDrain => DbSkillCategory::StaminaDrain,
            SkillCategory::VisionDebuff => DbSkillCategory::VisionDebuff,
            SkillCategory::Purple => DbSkillCategory::Purple,
            SkillCategory::Scenario => DbSkillCategory::Scenario,
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
    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

impl From<SkillOperator> for DbSkillOperator {
    fn from(o: SkillOperator) -> Self {
        match o {
            SkillOperator::Eq => DbSkillOperator::Eq,
            SkillOperator::NotEq => DbSkillOperator::NotEq,
            SkillOperator::Gt => DbSkillOperator::Gt,
            SkillOperator::GtEq => DbSkillOperator::GtEq,
            SkillOperator::Lt => DbSkillOperator::Lt,
            SkillOperator::LtEq => DbSkillOperator::LtEq,
        }
    }
}
