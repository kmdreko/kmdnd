use serde::{Deserialize, Serialize};

use crate::typedid::{TypedId, TypedIdMarker};

pub type CampaignId = TypedId<Campaign>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    #[serde(rename = "_id")]
    pub id: CampaignId,
    pub name: String,
}

impl TypedIdMarker for Campaign {
    fn tag() -> &'static str {
        "CPN"
    }
}
