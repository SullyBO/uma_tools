use crate::ids::{SkillId, UmaId};
use std::fmt::Display;

pub struct Uma {
    pub id: UmaId,
    pub name: String,
    pub subtitle: String,
    pub rarity: UmaRarity,
    pub base_stats: BaseStats,
    pub aptitudes: Aptitudes,
    pub skill_ids: Vec<SkillId>,
}

pub enum UmaRarity {
    R,
    SR,
    SSR,
}

impl Display for UmaRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::R => write!(f, "⭐"),
            Self::SR => write!(f, "⭐⭐"),
            Self::SSR => write!(f, "⭐⭐⭐"),
        }
    }
}

pub struct BaseStats {
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub guts: u32,
    pub wit: u32,
}

pub struct Aptitudes {
    pub surface: SurfaceAptitudes,
    pub distance: DistanceAptitudes,
    pub strategy: StrategyAptitudes,
}

pub struct SurfaceAptitudes {
    pub turf: AptitudeLevel,
    pub dirt: AptitudeLevel,
}

pub struct DistanceAptitudes {
    pub short: AptitudeLevel,
    pub mile: AptitudeLevel,
    pub medium: AptitudeLevel,
    pub long: AptitudeLevel,
}

pub struct StrategyAptitudes {
    pub front: AptitudeLevel,
    pub pace: AptitudeLevel,
    pub late: AptitudeLevel,
    pub end: AptitudeLevel,
}

pub enum AptitudeLevel {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
