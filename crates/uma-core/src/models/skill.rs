pub struct Skill {
    pub id: u32,
    pub name: String,
    pub ingame_description: String,
    pub category: SkillCategory,
    pub rarity: SkillRarity,
    pub sp_cost: u32,
    pub eval_points: u32,
}

pub struct CharacterSkill {
    pub skill_id: u32,
    pub acquisition: AcquisitionMethod,
}

pub enum AcquisitionMethod {
    Innate,
    Awakening,
    Event,
}

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
    RushDebuff,
    StaminaDrain,
    VisionDebuff,
}

#[derive(Copy, Clone)]
pub enum SkillRarity {
    Normal,
    Rare,
    Unique,
}
