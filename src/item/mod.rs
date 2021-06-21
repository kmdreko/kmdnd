use serde::{Deserialize, Serialize};

use crate::typedid::{TypedId, TypedIdMarker};

pub mod db;
pub mod endpoints;
pub use endpoints::*;

pub type ItemId = TypedId<Item>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "_id")]
    pub id: ItemId,
    pub name: String,
    pub weight: i32,
    pub value: i32,
    pub item_type: ItemType,
}

impl TypedIdMarker for Item {
    fn tag() -> &'static str {
        "ITM"
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ItemType {
    Weapon(Weapon),
    Armor(Armor),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Weapon {
    pub damage_amount: Dice,
    pub damage_type: DamageType,
    pub properties: Vec<WeaponProperty>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum WeaponProperty {
    Ammunition { normal_range: i32, long_range: i32 },
    Finesse,
    Heavy,
    Light,
    Loading,
    Range { normal_range: i32, long_range: i32 },
    Reach,
    Special,
    Thrown { normal_range: i32, long_range: i32 },
    TwoHanded,
    Versatile { two_handed_damage: Dice },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Dice {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Armor {}
