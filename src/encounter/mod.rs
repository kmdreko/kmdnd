use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::typedid::{TypedId, TypedIdMarker};

pub mod db;
pub mod endpoints;
pub use endpoints::*;

pub type EncounterId = TypedId<Encounter>;
pub type Round = i32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Encounter {
    #[serde(rename = "_id")]
    pub id: EncounterId,
    pub campaign_id: CampaignId,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
    pub character_ids: Vec<CharacterId>,
    pub state: EncounterState,
}

impl TypedIdMarker for Encounter {
    fn tag() -> &'static str {
        "ENC"
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EncounterState {
    Initiative,
    Turn {
        round: Round,
        character_id: CharacterId,
    },
    Finished,
}
