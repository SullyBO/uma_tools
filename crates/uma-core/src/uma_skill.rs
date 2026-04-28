use crate::ids::SkillId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillAcquisition {
    Unique,
    Innate,
    Awakening,
    Event,
    Evolution(SkillId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UmaSkill {
    pub id: SkillId,
    pub acquisition: SkillAcquisition,
}

impl SkillAcquisition {
    pub fn evolved_from(&self) -> Option<SkillId> {
        match self {
            SkillAcquisition::Evolution(id) => Some(*id),
            _ => None,
        }
    }
}
