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
pub mod manager;
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
        match *self {
            OperationType::Roll { roll, result } => Some((roll, result)),
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RollType {
    Initiative,
    SkillCheck(SkillType),
    AbilityCheck(AbilityType),
    Save(AbilityType),
    Hit,
    Damage,
}

impl Serialize for RollType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use AbilityType::*;
        use RollType::*;
        use SkillType::*;

        match self {
            Initiative => "INITIATIVE".serialize(serializer),
            SkillCheck(Acrobatics) => "ACROBATICS-CHECK".serialize(serializer),
            SkillCheck(AnimalHandling) => "ANIMAL-HANDLING-CHECK".serialize(serializer),
            SkillCheck(Arcana) => "ARCANA-CHECK".serialize(serializer),
            SkillCheck(Athletics) => "ATHLETICS-CHECK".serialize(serializer),
            SkillCheck(Deception) => "DECEPTION-CHECK".serialize(serializer),
            SkillCheck(History) => "HISTORY-CHECK".serialize(serializer),
            SkillCheck(Insight) => "INSIGHT-CHECK".serialize(serializer),
            SkillCheck(Intimidation) => "INTIMIDATION-CHECK".serialize(serializer),
            SkillCheck(Investigation) => "INVESTIGATION-CHECK".serialize(serializer),
            SkillCheck(Medicine) => "MEDICINE-CHECK".serialize(serializer),
            SkillCheck(Nature) => "NATURE-CHECK".serialize(serializer),
            SkillCheck(Perception) => "PERCEPTION-CHECK".serialize(serializer),
            SkillCheck(Performance) => "PERFORMANCE-CHECK".serialize(serializer),
            SkillCheck(Persuasion) => "PERSUASION-CHECK".serialize(serializer),
            SkillCheck(Religion) => "RELIGION-CHECK".serialize(serializer),
            SkillCheck(SleightOfHand) => "SLEIGHT-OF-HAND-CHECK".serialize(serializer),
            SkillCheck(Stealth) => "STEALTH-CHECK".serialize(serializer),
            SkillCheck(Survival) => "SURVIVAL-CHECK".serialize(serializer),
            AbilityCheck(Strength) => "STRENGTH-CHECK".serialize(serializer),
            AbilityCheck(Dexterity) => "DEXTERITY-CHECK".serialize(serializer),
            AbilityCheck(Constitution) => "CONSTITUTION-CHECK".serialize(serializer),
            AbilityCheck(Intelligence) => "INTELLIGENCE-CHECK".serialize(serializer),
            AbilityCheck(Wisdom) => "WISDOM-CHECK".serialize(serializer),
            AbilityCheck(Charisma) => "CHARISMA-CHECK".serialize(serializer),
            Save(Strength) => "STRENGTH-SAVE".serialize(serializer),
            Save(Dexterity) => "DEXTERITY-SAVE".serialize(serializer),
            Save(Constitution) => "CONSTITUTION-SAVE".serialize(serializer),
            Save(Intelligence) => "INTELLIGENCE-SAVE".serialize(serializer),
            Save(Wisdom) => "WISDOM-SAVE".serialize(serializer),
            Save(Charisma) => "CHARISMA-SAVE".serialize(serializer),
            Hit => "HIT".serialize(serializer),
            Damage => "DAMAGE".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for RollType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        use AbilityType::*;
        use RollType::*;
        use SkillType::*;

        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "INITIATIVE" => Ok(Initiative),
            "ACROBATICS-CHECK" => Ok(SkillCheck(Acrobatics)),
            "ANIMAL-HANDLING-CHECK" => Ok(SkillCheck(AnimalHandling)),
            "ARCANA-CHECK" => Ok(SkillCheck(Arcana)),
            "ATHLETICS-CHECK" => Ok(SkillCheck(Athletics)),
            "DECEPTION-CHECK" => Ok(SkillCheck(Deception)),
            "HISTORY-CHECK" => Ok(SkillCheck(History)),
            "INSIGHT-CHECK" => Ok(SkillCheck(Insight)),
            "INTIMIDATION-CHECK" => Ok(SkillCheck(Intimidation)),
            "INVESTIGATION-CHECK" => Ok(SkillCheck(Investigation)),
            "MEDICINE-CHECK" => Ok(SkillCheck(Medicine)),
            "NATURE-CHECK" => Ok(SkillCheck(Nature)),
            "PERCEPTION-CHECK" => Ok(SkillCheck(Perception)),
            "PERFORMANCE-CHECK" => Ok(SkillCheck(Performance)),
            "PERSUASION-CHECK" => Ok(SkillCheck(Persuasion)),
            "RELIGION-CHECK" => Ok(SkillCheck(Religion)),
            "SLEIGHT-OF-HAND-CHECK" => Ok(SkillCheck(SleightOfHand)),
            "STEALTH-CHECK" => Ok(SkillCheck(Stealth)),
            "SURVIVAL-CHECK" => Ok(SkillCheck(Survival)),
            "STRENGTH-CHECK" => Ok(AbilityCheck(Strength)),
            "DEXTERITY-CHECK" => Ok(AbilityCheck(Dexterity)),
            "CONSTITUTION-CHECK" => Ok(AbilityCheck(Constitution)),
            "INTELLIGENCE-CHECK" => Ok(AbilityCheck(Intelligence)),
            "WISDOM-CHECK" => Ok(AbilityCheck(Wisdom)),
            "CHARISMA-CHECK" => Ok(AbilityCheck(Charisma)),
            "STRENGTH-SAVE" => Ok(Save(Strength)),
            "DEXTERITY-SAVE" => Ok(Save(Dexterity)),
            "CONSTITUTION-SAVE" => Ok(Save(Constitution)),
            "INTELLIGENCE-SAVE" => Ok(Save(Intelligence)),
            "WISDOM-SAVE" => Ok(Save(Wisdom)),
            "CHARISMA-SAVE" => Ok(Save(Charisma)),
            "HIT" => Ok(Hit),
            "DAMAGE" => Ok(Damage),
            _ => Err(D::Error::custom("did not match")),
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
