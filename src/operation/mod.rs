use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::character::Position;
use crate::encounter::EncounterId;
use crate::item::ItemId;
use crate::typedid::{TypedId, TypedIdMarker};

pub mod db;
pub mod endpoints;
pub use endpoints::*;

pub type OperationId = TypedId<Operation>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(rename = "_id")]
    pub id: OperationId,
    pub campaign_id: CampaignId,
    pub encounter_id: Option<EncounterId>,
    pub character_id: CharacterId,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
    pub operation_type: OperationType,
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
    Roll { roll: Roll, result: i32 },
}

impl OperationType {
    pub fn as_roll(&self) -> Option<(Roll, i32)> {
        match self {
            &OperationType::Roll { roll, result } => Some((roll, result)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Roll {
    Initiative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    to_position: Position,
    feet: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Action {
    Melee,
    Attack(Attack),
    CastSpell,
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
pub struct Attack {
    targets: Vec<AttackTarget>,
    weapon_id: ItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackTarget {
    character_id: CharacterId,
    hit_roll: i32,
    damage: Option<i32>,
}
