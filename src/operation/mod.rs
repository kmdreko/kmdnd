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
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RollType {
    Initiative,
    Check(AbilityOrSkillType),
    Save(AbilityOrSkillType),
    Hit,
    Damage,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbilityOrSkillType {
    Ability(AbilityType),
    Skill(SkillType),
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
