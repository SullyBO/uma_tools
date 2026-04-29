use crate::ids::SkillId;
#[derive(Debug)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub ingame_description: String,
    pub category: Category,
    pub rarity: Rarity,
    pub sp_cost: u32,
    pub effects: Vec<Effect>,
    pub is_jp_only: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum Category {
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
    Unique,
    UniqueRecovery,
    Zenkai,
}

#[derive(Copy, Clone, Debug)]
pub enum Rarity {
    Normal,
    Rare,
    Unique,
    Evolution,
}

#[derive(Debug)]
pub struct Effect {
    pub effects: Vec<EffectType>,
    pub conditions: Vec<Condition>,
    pub preconditions: Vec<Condition>,
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

#[derive(Debug)]
pub struct Condition {
    pub cond_key: String,
    pub operator: Operator,
    pub cond_val: String,
    pub is_or: bool,
}

#[derive(Debug)]
pub struct ConditionType {
    pub cond_key: String,
    pub description: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EffectType {
    SpeedUp(i32),
    StaminaUp(i32),
    PowerUp(i32),
    GutsUp(i32),
    WitUp(i32),
    AllStatsUp(i32),
    CurrentSpeedUp(i32),
    CurrentSpeedDown(i32),
    TargetSpeedUp(i32),
    ZenkaiSpurtAcceleration(i32),
    AccelerationUp(i32),
    StaminaRecovery(i32),
    FieldOfViewUp(i32),
    LaneChangeSpeed(i32),
    ChangeLane(i32),
    StartReactionImprovement(i32),
    StartDelayAdded(i32),
    RushTimeIncrease(i32),
    RushChanceDecrease(i32),
    RunawaySkill,
    DebuffImmunity,
    ActivateRelatedSkillsOnAllUma,
    UseRandomRareSkills(i32),
    EvolvedSkillDurationUp(i32),
}

impl EffectType {
    /// Construct from raw JSON type id and raw value, normalized to base 10000.
    pub fn from_raw(type_id: u64, raw_value: i64) -> Option<Self> {
        let v10 = |divisor: i64| (raw_value * 10000 / divisor) as i32;

        match type_id {
            1 => Some(Self::SpeedUp(v10(1000))),
            2 => Some(Self::StaminaUp(v10(1000))),
            3 => Some(Self::PowerUp(v10(1000))),
            4 => Some(Self::GutsUp(v10(1000))),
            5 => Some(Self::WitUp(v10(1000))),
            6 => Some(Self::RunawaySkill),
            8 => Some(Self::FieldOfViewUp(v10(1000))),
            9 => Some(Self::StaminaRecovery(v10(1000))),
            10 => Some(Self::StartReactionImprovement(v10(10000))),
            13 => Some(Self::RushTimeIncrease(v10(10000))),
            14 => Some(Self::StartDelayAdded(v10(10000))),
            21 => Some(Self::CurrentSpeedDown(v10(10000))),
            22 => Some(Self::CurrentSpeedUp(v10(10000))),
            27 => Some(Self::TargetSpeedUp(v10(10000))),
            28 => Some(Self::LaneChangeSpeed(v10(1000))),
            29 => Some(Self::RushChanceDecrease(v10(10000))),
            31 => Some(Self::AccelerationUp(v10(1000))),
            32 => Some(Self::AllStatsUp(v10(10000))),
            35 => Some(Self::ChangeLane(v10(100))),
            37 => Some(Self::UseRandomRareSkills(v10(10000))),
            38 => Some(Self::DebuffImmunity),
            41 => Some(Self::ActivateRelatedSkillsOnAllUma),
            42 => Some(Self::EvolvedSkillDurationUp(v10(1000))),
            48 => Some(Self::ZenkaiSpurtAcceleration(v10(10000))),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::SpeedUp(_) => "Speed Up",
            Self::StaminaUp(_) => "Stamina Up",
            Self::PowerUp(_) => "Power Up",
            Self::GutsUp(_) => "Guts Up",
            Self::WitUp(_) => "Wit Up",
            Self::AllStatsUp(_) => "All Stats Up",
            Self::CurrentSpeedUp(_) => "Current Speed Up",
            Self::CurrentSpeedDown(_) => "Current Speed Down",
            Self::TargetSpeedUp(_) => "Increase Target Speed",
            Self::ZenkaiSpurtAcceleration(_) => "Zenkai Spurt Acceleration",
            Self::AccelerationUp(_) => "Acceleration Up",
            Self::StaminaRecovery(_) => "Stamina Recovery",
            Self::FieldOfViewUp(_) => "Field of View Up",
            Self::LaneChangeSpeed(_) => "Lane Change Speed",
            Self::ChangeLane(_) => "Change Lane",
            Self::StartReactionImprovement(_) => "Start Reaction Improvement",
            Self::StartDelayAdded(_) => "Start Delay Added",
            Self::RushTimeIncrease(_) => "Rush Time Increase",
            Self::RushChanceDecrease(_) => "Rush Chance Decrease",
            Self::UseRandomRareSkills(_) => "Use Random Rare Skills",
            Self::EvolvedSkillDurationUp(_) => "Evolved Skill Duration Up",
            Self::RunawaySkill => "Runaway",
            Self::DebuffImmunity => "Debuff Immunity",
            Self::ActivateRelatedSkillsOnAllUma => "Activate Related Skills on All Uma",
        }
    }

    pub fn value(&self) -> Option<i32> {
        match self {
            Self::RunawaySkill | Self::DebuffImmunity | Self::ActivateRelatedSkillsOnAllUma => None,
            Self::SpeedUp(v)
            | Self::StaminaUp(v)
            | Self::PowerUp(v)
            | Self::GutsUp(v)
            | Self::WitUp(v)
            | Self::AllStatsUp(v)
            | Self::CurrentSpeedUp(v)
            | Self::CurrentSpeedDown(v)
            | Self::TargetSpeedUp(v)
            | Self::ZenkaiSpurtAcceleration(v)
            | Self::AccelerationUp(v)
            | Self::StaminaRecovery(v)
            | Self::FieldOfViewUp(v)
            | Self::LaneChangeSpeed(v)
            | Self::ChangeLane(v)
            | Self::StartReactionImprovement(v)
            | Self::StartDelayAdded(v)
            | Self::RushTimeIncrease(v)
            | Self::RushChanceDecrease(v)
            | Self::UseRandomRareSkills(v)
            | Self::EvolvedSkillDurationUp(v) => Some(*v),
        }
    }
}

impl std::fmt::Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_val(v: i32) -> String {
            if v % 10000 == 0 {
                format!("{}", v / 10000)
            } else if v % 1000 == 0 {
                format!("{:.1}", v as f64 / 10000.0)
            } else if v % 100 == 0 {
                format!("{:.2}", v as f64 / 10000.0)
            } else {
                format!("{:.4}", v as f64 / 10000.0)
            }
        }

        match self {
            Self::SpeedUp(v) => write!(f, "Speed +{}", fmt_val(*v)),
            Self::StaminaUp(v) => write!(f, "Stamina +{}", fmt_val(*v)),
            Self::PowerUp(v) => write!(f, "Power +{}", fmt_val(*v)),
            Self::GutsUp(v) => write!(f, "Guts +{}", fmt_val(*v)),
            Self::WitUp(v) => write!(f, "Wit +{}", fmt_val(*v)),
            Self::AllStatsUp(v) => write!(f, "All Stats +{}", fmt_val(*v)),
            Self::CurrentSpeedUp(v) => write!(f, "Current Speed +{}", fmt_val(*v)),
            Self::CurrentSpeedDown(v) => write!(f, "Current Speed {}", fmt_val(*v)),
            Self::TargetSpeedUp(v) => write!(f, "Target Speed +{}", fmt_val(*v)),
            Self::ZenkaiSpurtAcceleration(v) => write!(f, "Zenkai Spurt Accel +{}", fmt_val(*v)),
            Self::AccelerationUp(v) => write!(f, "Acceleration +{}", fmt_val(*v)),
            Self::StaminaRecovery(v) => write!(f, "Stamina Recovery +{}", fmt_val(*v)),
            Self::FieldOfViewUp(v) => write!(f, "Field of View +{}", fmt_val(*v)),
            Self::LaneChangeSpeed(v) => write!(f, "Lane Change Speed +{}", fmt_val(*v)),
            Self::ChangeLane(v) => write!(f, "Change Lane {}", fmt_val(*v)),
            Self::StartReactionImprovement(v) => write!(f, "Start Reaction +{}", fmt_val(*v)),
            Self::StartDelayAdded(v) => write!(f, "Start Delay +{}", fmt_val(*v)),
            Self::RushTimeIncrease(v) => write!(f, "Rush Time +{}", fmt_val(*v)),
            Self::RushChanceDecrease(v) => write!(f, "Rush Chance {}", fmt_val(*v)),
            Self::UseRandomRareSkills(v) => write!(f, "Random Rare Skills +{}", fmt_val(*v)),
            Self::EvolvedSkillDurationUp(v) => write!(f, "Evolved Skill Duration +{}", fmt_val(*v)),
            Self::RunawaySkill => write!(f, "Runaway"),
            Self::DebuffImmunity => write!(f, "Debuff Immunity"),
            Self::ActivateRelatedSkillsOnAllUma => write!(f, "Activate Related Skills on All Uma"),
        }
    }
}
