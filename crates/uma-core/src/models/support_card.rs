use crate::ids::SupportCardId;
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub struct SupportCard {
    pub id: SupportCardId,
    pub char_id: u32,
    pub char_name: String,
    pub title: String,
    pub card_type: CardType,
    pub rarity: Rarity,
    pub is_welfare: bool,
    pub release_en: Option<NaiveDate>,
    pub is_predicted_date: bool,
    pub unique_effect: Option<String>,
    pub effects: Vec<EffectValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rarity {
    R,
    SR,
    SSR,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    Speed,
    Stamina,
    Power,
    Guts,
    Wit,
    Friend,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LbValues {
    pub lb0: Option<i32>,
    pub lb1: Option<i32>,
    pub lb2: Option<i32>,
    pub lb3: Option<i32>,
    pub mlb: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectValue {
    /// Effect ID 1
    FriendshipBonus(LbValues),
    /// Effect ID 2
    MoodEffect(LbValues),
    /// Effect ID 3
    SpeedBonus(LbValues),
    /// Effect ID 4
    StaminaBonus(LbValues),
    /// Effect ID 5
    PowerBonus(LbValues),
    /// Effect ID 6
    GutsBonus(LbValues),
    /// Effect ID 7
    WitBonus(LbValues),
    /// Effect ID 8
    TrainingEffectiveness(LbValues),
    /// Effect ID 9
    InitialSpeed(LbValues),
    /// Effect ID 10
    InitialStamina(LbValues),
    /// Effect ID 11
    InitialPower(LbValues),
    /// Effect ID 12
    InitialGuts(LbValues),
    /// Effect ID 13
    InitialWit(LbValues),
    /// Effect ID 14
    InitialFriendshipGauge(LbValues),
    /// Effect ID 15
    RaceBonus(LbValues),
    /// Effect ID 16
    FanBonus(LbValues),
    /// Effect ID 17
    HintLevels(LbValues),
    /// Effect ID 18
    HintFrequency(LbValues),
    /// Effect ID 19
    SpecialtyPriority(LbValues),
    /// Effect ID 20 — inactive
    MaxSpeed(LbValues),
    /// Effect ID 21 — inactive
    MaxStamina(LbValues),
    /// Effect ID 22 — inactive
    MaxPower(LbValues),
    /// Effect ID 23 — inactive
    MaxGuts(LbValues),
    /// Effect ID 24 — inactive
    MaxWit(LbValues),
    /// Effect ID 25
    EventRecovery(LbValues),
    /// Effect ID 26
    EventEffectiveness(LbValues),
    /// Effect ID 27
    FailureProtection(LbValues),
    /// Effect ID 28
    EnergyCostReduction(LbValues),
    /// Effect ID 29 — inactive
    MinigameEffectiveness(LbValues),
    /// Effect ID 30
    SkillPointBonus(LbValues),
    /// Effect ID 31
    WitFriendshipRecovery(LbValues),
    /// Effect ID 32
    InitialSkillPoints(LbValues),
    /// Effect ID 33
    HintQuantityBonus(LbValues),
    /// Effect ID 41
    AllStatsBonus(LbValues),
}

impl EffectValue {
    pub fn from_id(id: u32, values: LbValues) -> Option<Self> {
        match id {
            1 => Some(Self::FriendshipBonus(values)),
            2 => Some(Self::MoodEffect(values)),
            3 => Some(Self::SpeedBonus(values)),
            4 => Some(Self::StaminaBonus(values)),
            5 => Some(Self::PowerBonus(values)),
            6 => Some(Self::GutsBonus(values)),
            7 => Some(Self::WitBonus(values)),
            8 => Some(Self::TrainingEffectiveness(values)),
            9 => Some(Self::InitialSpeed(values)),
            10 => Some(Self::InitialStamina(values)),
            11 => Some(Self::InitialPower(values)),
            12 => Some(Self::InitialGuts(values)),
            13 => Some(Self::InitialWit(values)),
            14 => Some(Self::InitialFriendshipGauge(values)),
            15 => Some(Self::RaceBonus(values)),
            16 => Some(Self::FanBonus(values)),
            17 => Some(Self::HintLevels(values)),
            18 => Some(Self::HintFrequency(values)),
            19 => Some(Self::SpecialtyPriority(values)),
            20 => Some(Self::MaxSpeed(values)),
            21 => Some(Self::MaxStamina(values)),
            22 => Some(Self::MaxPower(values)),
            23 => Some(Self::MaxGuts(values)),
            24 => Some(Self::MaxWit(values)),
            25 => Some(Self::EventRecovery(values)),
            26 => Some(Self::EventEffectiveness(values)),
            27 => Some(Self::FailureProtection(values)),
            28 => Some(Self::EnergyCostReduction(values)),
            29 => Some(Self::MinigameEffectiveness(values)),
            30 => Some(Self::SkillPointBonus(values)),
            31 => Some(Self::WitFriendshipRecovery(values)),
            32 => Some(Self::InitialSkillPoints(values)),
            33 => Some(Self::HintQuantityBonus(values)),
            41 => Some(Self::AllStatsBonus(values)),
            unknown => {
                log::warn!("Unknown effect ID {unknown} — add variant to EffectValue");
                None
            }
        }
    }
}
