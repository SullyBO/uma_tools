use crate::ids::UmaId;
use crate::uma_skill::UmaSkill;
use std::fmt::Display;

#[derive(Debug)]
pub struct Uma {
    pub id: UmaId,
    pub name: String,
    pub subtitle: String,
    pub rarity: UmaRarity,
    pub base_stats: BaseStats,
    pub growth_rates: GrowthRates,
    pub aptitudes: Aptitudes,
    pub skill_list: Vec<UmaSkill>,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub struct BaseStats {
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub guts: u32,
    pub wit: u32,
}

#[derive(Debug)]
pub struct GrowthRates {
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub guts: u32,
    pub wit: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Aptitudes {
    pub surface: SurfaceAptitudes,
    pub distance: DistanceAptitudes,
    pub strategy: StrategyAptitudes,
}

#[derive(Debug, Copy, Clone)]
pub struct SurfaceAptitudes {
    pub turf: AptitudeLevel,
    pub dirt: AptitudeLevel,
}

#[derive(Debug, Copy, Clone)]
pub struct DistanceAptitudes {
    pub short: AptitudeLevel,
    pub mile: AptitudeLevel,
    pub medium: AptitudeLevel,
    pub long: AptitudeLevel,
}

#[derive(Debug, Copy, Clone)]
pub struct StrategyAptitudes {
    pub front: AptitudeLevel,
    pub pace: AptitudeLevel,
    pub late: AptitudeLevel,
    pub end: AptitudeLevel,
}

#[derive(Debug, Copy, Clone)]
pub enum AptitudeLevel {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
