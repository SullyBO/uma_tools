pub struct Skill {
    pub id: u32,
    pub name: String,
    pub acquisition: AcquisitionMethod,
    pub ingame_description: String,
    pub detailed_description: String,
    pub category: SkillCategory,
    pub rarity: SkillRarity,
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

pub enum SkillRarity {
    Normal,
    Rare,
    Unique,
}
