use std::future;

use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::database::Database;
use crate::error::Error;
use crate::item::{ArmorType, ItemId};
use crate::operation::{AbilityType, SkillType};
use crate::typedid::{TypedId, TypedIdMarker};
use crate::user::UserId;

pub mod db;
pub mod endpoints;
pub mod manager;
pub mod race;
pub use endpoints::*;

use self::race::{Race, RacialTrait};

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
    pub position: Option<Position>,
    pub current_hit_points: i32,
    pub maximum_hit_points: i32,
    pub race: Race,
    pub racial_traits: Vec<RacialTrait>,
    pub proficiencies: Proficiencies,
    // conditions: Vec<Condition>,
}

impl Character {
    pub async fn recalculate_stats(&mut self, db: &dyn Database) -> Result<(), Error> {
        let items: Vec<_> = stream::iter(&self.equipment)
            .filter(|entry| future::ready(entry.equiped))
            .then(|entry| db.items().fetch_item_by_id(entry.item_id))
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

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn distance(&self, other: &Position) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;

        f32::sqrt(x * x + y * y + z * z)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Proficiencies {
    pub armor: Vec<ArmorType>,
    pub tool: Vec<ToolType>,
    pub saving_throws: Vec<AbilityType>,
    pub skills: Vec<SkillType>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RollModifier {
    Advantage,
    Normal,
    Disadvantage,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ToolType {
    // Artisan Tools
    AlchemistsSupplies,
    BrewersSupplies,
    CalligrapherSupplies,
    CarpentersTools,
    CartographersTools,
    CobblersTools,
    CooksUtensils,
    GlassblowersTools,
    JewelersTools,
    LeatherworkersTools,
    MasonsTools,
    PainterTupplies,
    PottersTools,
    SmithsTools,
    TinkersTools,
    WeaversTools,
    WoodcarversTools,

    DisguiseKit,

    ForgeryKit,

    // Gaming Sets
    DiceSet,
    PlayingCardSet,

    HerbalismKit,

    // Musical Instruments
    Bagpipes,
    Drum,
    Dulcimer,
    Flute,
    Lute,
    Lyre,
    Horn,
    PanFlute,
    Shawm,
    Viol,

    NavigatorTools,

    PoisonerKit,
    TheivesTools,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Language {
    Common,
    Dwarvish,
    Elvish,
    Giant,
    Gnomish,
    Goblin,
    Halfling,
    Orc,

    Abyssal,
    Celestial,
    Draconic,
    DeepSpeech,
    Infernal,
    Primordial,
    Sylvan,
    Undercommon,
}
