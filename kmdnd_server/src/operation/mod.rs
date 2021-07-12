use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::character::Position;
use crate::encounter::EncounterId;
use crate::encounter::EncounterState;
use crate::typedid::{TypedId, TypedIdMarker};
use crate::violations::Violation;

pub mod attack;
pub mod db;
pub mod endpoints;
pub mod spell;
pub use endpoints::*;

use attack::Attack;
use spell::Cast;

pub type OperationId = TypedId<Operation>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(rename = "_id")]
    pub id: OperationId,
    pub campaign_id: CampaignId,
    pub encounter_id: Option<EncounterId>,
    pub encounter_state: Option<EncounterState>,
    pub character_id: CharacterId,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
    pub operation_type: OperationType,
    pub interactions: Vec<Interaction>,
    pub legality: Legality,
}

impl TypedIdMarker for Operation {
    fn tag() -> &'static str {
        "OPR"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum OperationType {
    Move(Move),
    Action(Action),
    Bonus { name: String },
    Roll { roll: RollType, result: i32 },
}

impl OperationType {
    pub fn as_roll(&self) -> Option<(RollType, i32)> {
        match self {
            &OperationType::Roll { roll, result } => Some((roll, result)),
            _ => None,
        }
    }

    pub fn as_move(&self) -> Option<&Move> {
        match self {
            OperationType::Move(mov) => Some(mov),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RollType {
    Initiative,
    StrengthCheck,
    DexterityCheck,
    ConstitutionCheck,
    IntelligenceCheck,
    WisdomCheck,
    CharismaCheck,
    AcrobaticsCheck,
    AnimalHandlingCheck,
    ArcanaCheck,
    AthleticsCheck,
    DeceptionCheck,
    HistoryCheck,
    InsightCheck,
    IntimidationCheck,
    InvestigationCheck,
    MedicineCheck,
    NatureCheck,
    PerceptionCheck,
    PerformanceCheck,
    PersuasionCheck,
    ReligionCheck,
    SleightOfHandCheck,
    StealthCheck,
    SurvivalCheck,
    StrengthSave,
    DexteritySave,
    ConstitutionSave,
    IntelligenceSave,
    WisdomSave,
    CharismaSave,
    Hit,
    Damage,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RollCategory {
    Initiative,
    SkillCheck(SkillType),
    AbilityCheck(AbilityType),
    Save(AbilityType),
    Hit,
    Damage,
}

impl From<RollType> for RollCategory {
    fn from(roll: RollType) -> Self {
        match roll {
            RollType::Initiative => RollCategory::Initiative,
            RollType::StrengthCheck => RollCategory::AbilityCheck(AbilityType::Strength),
            RollType::DexterityCheck => RollCategory::AbilityCheck(AbilityType::Dexterity),
            RollType::ConstitutionCheck => RollCategory::AbilityCheck(AbilityType::Constitution),
            RollType::IntelligenceCheck => RollCategory::AbilityCheck(AbilityType::Intelligence),
            RollType::WisdomCheck => RollCategory::AbilityCheck(AbilityType::Wisdom),
            RollType::CharismaCheck => RollCategory::AbilityCheck(AbilityType::Charisma),
            RollType::AcrobaticsCheck => RollCategory::SkillCheck(SkillType::Acrobatics),
            RollType::AnimalHandlingCheck => RollCategory::SkillCheck(SkillType::AnimalHandling),
            RollType::ArcanaCheck => RollCategory::SkillCheck(SkillType::Arcana),
            RollType::AthleticsCheck => RollCategory::SkillCheck(SkillType::Athletics),
            RollType::DeceptionCheck => RollCategory::SkillCheck(SkillType::Deception),
            RollType::HistoryCheck => RollCategory::SkillCheck(SkillType::History),
            RollType::InsightCheck => RollCategory::SkillCheck(SkillType::Insight),
            RollType::IntimidationCheck => RollCategory::SkillCheck(SkillType::Intimidation),
            RollType::InvestigationCheck => RollCategory::SkillCheck(SkillType::Investigation),
            RollType::MedicineCheck => RollCategory::SkillCheck(SkillType::Medicine),
            RollType::NatureCheck => RollCategory::SkillCheck(SkillType::Nature),
            RollType::PerceptionCheck => RollCategory::SkillCheck(SkillType::Perception),
            RollType::PerformanceCheck => RollCategory::SkillCheck(SkillType::Performance),
            RollType::PersuasionCheck => RollCategory::SkillCheck(SkillType::Persuasion),
            RollType::ReligionCheck => RollCategory::SkillCheck(SkillType::Religion),
            RollType::SleightOfHandCheck => RollCategory::SkillCheck(SkillType::SleightOfHand),
            RollType::StealthCheck => RollCategory::SkillCheck(SkillType::Stealth),
            RollType::SurvivalCheck => RollCategory::SkillCheck(SkillType::Survival),
            RollType::StrengthSave => RollCategory::Save(AbilityType::Strength),
            RollType::DexteritySave => RollCategory::Save(AbilityType::Dexterity),
            RollType::ConstitutionSave => RollCategory::Save(AbilityType::Constitution),
            RollType::IntelligenceSave => RollCategory::Save(AbilityType::Intelligence),
            RollType::WisdomSave => RollCategory::Save(AbilityType::Wisdom),
            RollType::CharismaSave => RollCategory::Save(AbilityType::Charisma),
            RollType::Hit => RollCategory::Hit,
            RollType::Damage => RollCategory::Damage,
        }
    }
}

impl Into<RollType> for RollCategory {
    fn into(self) -> RollType {
        match self {
            RollCategory::Initiative => RollType::Initiative,
            RollCategory::AbilityCheck(AbilityType::Strength) => RollType::StrengthCheck,
            RollCategory::AbilityCheck(AbilityType::Dexterity) => RollType::DexterityCheck,
            RollCategory::AbilityCheck(AbilityType::Constitution) => RollType::ConstitutionCheck,
            RollCategory::AbilityCheck(AbilityType::Intelligence) => RollType::IntelligenceCheck,
            RollCategory::AbilityCheck(AbilityType::Wisdom) => RollType::WisdomCheck,
            RollCategory::AbilityCheck(AbilityType::Charisma) => RollType::CharismaCheck,
            RollCategory::SkillCheck(SkillType::Acrobatics) => RollType::AcrobaticsCheck,
            RollCategory::SkillCheck(SkillType::AnimalHandling) => RollType::AnimalHandlingCheck,
            RollCategory::SkillCheck(SkillType::Arcana) => RollType::ArcanaCheck,
            RollCategory::SkillCheck(SkillType::Athletics) => RollType::AthleticsCheck,
            RollCategory::SkillCheck(SkillType::Deception) => RollType::DeceptionCheck,
            RollCategory::SkillCheck(SkillType::History) => RollType::HistoryCheck,
            RollCategory::SkillCheck(SkillType::Insight) => RollType::InsightCheck,
            RollCategory::SkillCheck(SkillType::Intimidation) => RollType::IntimidationCheck,
            RollCategory::SkillCheck(SkillType::Investigation) => RollType::InvestigationCheck,
            RollCategory::SkillCheck(SkillType::Medicine) => RollType::MedicineCheck,
            RollCategory::SkillCheck(SkillType::Nature) => RollType::NatureCheck,
            RollCategory::SkillCheck(SkillType::Perception) => RollType::PerceptionCheck,
            RollCategory::SkillCheck(SkillType::Performance) => RollType::PerformanceCheck,
            RollCategory::SkillCheck(SkillType::Persuasion) => RollType::PersuasionCheck,
            RollCategory::SkillCheck(SkillType::Religion) => RollType::ReligionCheck,
            RollCategory::SkillCheck(SkillType::SleightOfHand) => RollType::SleightOfHandCheck,
            RollCategory::SkillCheck(SkillType::Stealth) => RollType::StealthCheck,
            RollCategory::SkillCheck(SkillType::Survival) => RollType::SurvivalCheck,
            RollCategory::Save(AbilityType::Strength) => RollType::StrengthSave,
            RollCategory::Save(AbilityType::Dexterity) => RollType::DexteritySave,
            RollCategory::Save(AbilityType::Constitution) => RollType::ConstitutionSave,
            RollCategory::Save(AbilityType::Intelligence) => RollType::IntelligenceSave,
            RollCategory::Save(AbilityType::Wisdom) => RollType::WisdomSave,
            RollCategory::Save(AbilityType::Charisma) => RollType::CharismaSave,
            RollCategory::Hit => RollType::Hit,
            RollCategory::Damage => RollType::Damage,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum AbilityType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum SkillType {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}

impl SkillType {
    pub fn ability(&self) -> AbilityType {
        match self {
            SkillType::Acrobatics => AbilityType::Dexterity,
            SkillType::AnimalHandling => AbilityType::Wisdom,
            SkillType::Arcana => AbilityType::Intelligence,
            SkillType::Athletics => AbilityType::Strength,
            SkillType::Deception => AbilityType::Charisma,
            SkillType::History => AbilityType::Intelligence,
            SkillType::Insight => AbilityType::Wisdom,
            SkillType::Intimidation => AbilityType::Charisma,
            SkillType::Investigation => AbilityType::Intelligence,
            SkillType::Medicine => AbilityType::Wisdom,
            SkillType::Nature => AbilityType::Intelligence,
            SkillType::Perception => AbilityType::Wisdom,
            SkillType::Performance => AbilityType::Charisma,
            SkillType::Persuasion => AbilityType::Charisma,
            SkillType::Religion => AbilityType::Intelligence,
            SkillType::SleightOfHand => AbilityType::Dexterity,
            SkillType::Stealth => AbilityType::Dexterity,
            SkillType::Survival => AbilityType::Wisdom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    to_position: Position,
    feet: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Action {
    Attack(Attack),
    CastSpell(Cast),
    Dash,
    Disengage,
    Dodge,
    Help,
    Hide,
    Ready,
    Search,
    UseObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum SpellTarget {
    Creature { character_id: CharacterId },
    Position { position: Position },
    None,
}

pub type InteractionId = TypedId<Interaction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    id: InteractionId,
    character_id: CharacterId,
    roll_type: RollType,
    result: Option<i32>,
}

impl TypedIdMarker for Interaction {
    fn tag() -> &'static str {
        "ITR"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Legality {
    Legal,
    IllegalPending { violations: Vec<Violation> },
    IllegalApproved { violations: Vec<Violation> },
}
