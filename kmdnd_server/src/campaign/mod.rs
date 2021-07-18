use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::typedid::{TypedId, TypedIdMarker};

pub mod db;
pub mod endpoints;
pub mod manager;
pub use endpoints::*;

pub type CampaignId = TypedId<Campaign>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    #[serde(rename = "_id")]
    pub id: CampaignId,
    pub name: String,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
}

impl TypedIdMarker for Campaign {
    fn tag() -> &'static str {
        "CPN"
    }
}
