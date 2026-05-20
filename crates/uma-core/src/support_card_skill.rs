use crate::ids::SkillId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SupportCardSkill {
    pub id: SkillId,
    pub acquisition: HintAcquisition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HintAcquisition {
    Event,
    Hint,
}
