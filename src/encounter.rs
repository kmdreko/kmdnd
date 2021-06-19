use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::typedid::{TypedId, TypedIdMarker};

pub type EncounterId = TypedId<Encounter>;
pub type Round = u32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Encounter {
    #[serde(rename = "_id")]
    pub id: EncounterId,
    pub campaign_id: CampaignId,
    pub created_at: DateTime<Utc>,
    pub character_ids: Vec<CharacterId>,
    pub state: EncounterState,
}

impl TypedIdMarker for Encounter {
    fn tag() -> &'static str {
        "ENC"
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum EncounterState {
    Initiative,
    Turn {
        round: Round,
        character_id: CharacterId,
    },
    Finished,
}
