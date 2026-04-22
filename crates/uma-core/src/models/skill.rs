use crate::ids::SkillId;
#[derive(Debug)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub ingame_description: String,
    pub category: SkillCategory,
    pub rarity: SkillRarity,
    pub sp_cost: u32,
    pub eval_points: u32,
}

#[derive(Debug)]
pub enum SkillCategory {
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

#[derive(Copy, Clone, Debug)]
pub enum SkillRarity {
    Normal,
    Rare,
    Unique,
    Evolution,
}
