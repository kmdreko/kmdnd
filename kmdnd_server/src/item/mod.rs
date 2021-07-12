use serde::{Deserialize, Serialize};

use crate::character::Character;
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

impl ItemType {
    pub fn as_armor(&self) -> Option<&Armor> {
        match self {
            ItemType::Armor(armor) => Some(armor),
            _ => None,
        }
    }

    pub fn into_weapon(self) -> Option<Weapon> {
        match self {
            ItemType::Weapon(weapon) => Some(weapon),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Weapon {
    pub damage_amount: Dice,
    pub damage_type: DamageType,
    pub properties: Vec<WeaponProperty>,
}

impl Weapon {
    pub fn normal_range(&self) -> f32 {
        let mut melee_range = 5.0;
        for property in &self.properties {
            match property {
                WeaponProperty::Ammunition(range) | WeaponProperty::Thrown(range) => {
                    return range.normal as f32
                }
                WeaponProperty::Reach => {
                    melee_range += 5.0;
                }
                _ => {}
            }
        }

        melee_range
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum WeaponProperty {
    Ammunition(Range),
    Finesse,
    Heavy,
    Light,
    Loading,
    Reach,
    Special,
    Thrown(Range),
    TwoHanded,
    Versatile { two_handed_damage: Dice },
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Range {
    pub normal: i32,
    pub long: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
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
pub struct Armor {
    pub base_armor_class: i32,
    pub armor_type: ArmorType,
    pub strength_requirement: Option<i32>,
    pub stealth_disadvantage: bool,
}

impl Armor {
    pub fn effective_armor_class(&self, character: &Character) -> i32 {
        let ac_from_dex = match self.armor_type {
            ArmorType::Light => character.stats.abilities.dexterity_modifier(),
            ArmorType::Medium => i32::min(2, character.stats.abilities.dexterity_modifier()),
            ArmorType::Heavy => 0,
            ArmorType::Shield => 2,
        };

        self.base_armor_class + ac_from_dex
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
    Shield,
}
