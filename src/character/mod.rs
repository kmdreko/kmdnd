use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::typedid::{TypedId, TypedIdMarker};
use crate::user::UserId;

pub mod db;
pub mod endpoints;
pub use endpoints::*;

pub type CharacterId = TypedId<Character>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    #[serde(rename = "_id")]
    pub id: CharacterId,
    pub owner: CharacterOwner,
    pub name: String,
    // attributes: String,
    // items: Vec<Item>,
    // position: Option<(f32, f32)>,
    // health: u32,
    // effects: Vec<Effect>,
}

impl TypedIdMarker for Character {
    fn tag() -> &'static str {
        "CHR"
    }
}

// A character must have an owning User, Campaign, or both
#[derive(Clone, Debug)]
pub enum CharacterOwner {
    Campaign(CampaignId),
    User(UserId),
    UserInCampaign(UserId, CampaignId),
}

impl Serialize for CharacterOwner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Dummy<'a> {
            campaign_id: Option<&'a CampaignId>,
            user_id: Option<&'a UserId>,
        }

        let dummy = match self {
            CharacterOwner::Campaign(campaign_id) => Dummy {
                campaign_id: Some(campaign_id),
                user_id: None,
            },
            CharacterOwner::User(user_id) => Dummy {
                campaign_id: None,
                user_id: Some(user_id),
            },
            CharacterOwner::UserInCampaign(user_id, campaign_id) => Dummy {
                campaign_id: Some(campaign_id),
                user_id: Some(user_id),
            },
        };

        dummy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CharacterOwner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Dummy {
            campaign_id: Option<CampaignId>,
            user_id: Option<UserId>,
        }

        let dummy = Dummy::deserialize(deserializer)?;

        match (dummy.campaign_id, dummy.user_id) {
            (Some(campaign_id), None) => Ok(CharacterOwner::Campaign(campaign_id)),
            (None, Some(user_id)) => Ok(CharacterOwner::User(user_id)),
            (Some(campaign_id), Some(user_id)) => {
                Ok(CharacterOwner::UserInCampaign(user_id, campaign_id))
            }
            (None, None) => Err(<D::Error as serde::de::Error>::custom(
                "character must have a user or campaign",
            )),
        }
    }
}
