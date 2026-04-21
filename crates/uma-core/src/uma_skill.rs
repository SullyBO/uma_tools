use crate::ids::SkillId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillAcquisition {
    Unique,
    Innate,
    Awakening,
    Event,
    Evolution,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UmaSkill {
    pub id: SkillId,
    pub acquisition: SkillAcquisition,
}
