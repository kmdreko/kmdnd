use std::future;

use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::error::Error;
use crate::item::{self, ItemId};
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
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub modified_at: DateTime<Utc>,
    pub stats: CharacterStats,
    pub equipment: Vec<EquipmentEntry>,
    // position: Option<(f32, f32)>,
    // health: i32,
    // effects: Vec<Effect>,
}

impl Character {
    pub async fn recalculate_stats(&mut self, db: &Database) -> Result<(), Error> {
        let items: Vec<_> = stream::iter(&self.equipment)
            .filter(|entry| future::ready(entry.equiped))
            .then(|entry| item::db::fetch_item_by_id(db, entry.item_id))
            .try_collect()
            .await?;

        let armor: Vec<_> = items
            .iter()
            .filter_map(|item| item.as_ref())
            .filter_map(|item| item.item_type.as_armor())
            .collect();

        let mut armor_class = 10;
        for piece in armor {
            armor_class += piece.effective_armor_class(self);
        }

        self.stats.armor_class = armor_class;

        Ok(())
    }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharacterStats {
    pub abilities: CharacterAbilities,
    pub initiative: i32,
    pub speed: i32,
    pub armor_class: i32,
    pub proficiency_bonus: i32,
}

impl Default for CharacterStats {
    fn default() -> CharacterStats {
        CharacterStats {
            abilities: CharacterAbilities::default(),
            initiative: 0,
            speed: 30,
            armor_class: 10,
            proficiency_bonus: 1,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharacterAbilities {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl CharacterAbilities {
    pub fn dexterity_modifier(&self) -> i32 {
        (self.dexterity - 10) / 2
    }
}

impl Default for CharacterAbilities {
    fn default() -> CharacterAbilities {
        CharacterAbilities {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquipmentEntry {
    pub equiped: bool,
    pub quantity: i32,
    pub item_id: ItemId,
}
