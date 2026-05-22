use crate::ids::SkillId;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SupportCardSkill {
    pub skill_id: SkillId,
    pub acquisition: HintAcquisition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HintAcquisition {
    Event,
    Hint,
}

impl fmt::Display for HintAcquisition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HintAcquisition::Event => write!(f, "event"),
            HintAcquisition::Hint => write!(f, "hint"),
        }
    }
}
