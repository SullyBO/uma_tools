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

pub struct SkillEffect {
    pub skill_id: SkillId,
    pub stats: Vec<SkillEffectStat>,
    pub conditions: Vec<SkillCondition>,
}

pub struct SkillEffectStat {
    pub stat_key: String,
    pub stat_val: String,
}

pub enum SkillOperator {
    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

pub struct SkillCondition {
    pub is_precondition: bool,
    pub cond_key: String,
    pub operator: SkillOperator,
    pub cond_val: String,
}
