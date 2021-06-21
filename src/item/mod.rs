use rand::Rng;
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
    pub fn as_weapon(&self) -> Option<&Weapon> {
        match self {
            ItemType::Weapon(weapon) => Some(weapon),
            _ => None,
        }
    }

    pub fn as_armor(&self) -> Option<&Armor> {
        match self {
            ItemType::Armor(armor) => Some(armor),
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

impl Dice {
    pub fn roll(&self) -> i32 {
        match self {
            Dice::D4 => rand::thread_rng().gen_range(1..=4),
            Dice::D6 => rand::thread_rng().gen_range(1..=6),
            Dice::D8 => rand::thread_rng().gen_range(1..=8),
            Dice::D10 => rand::thread_rng().gen_range(1..=10),
            Dice::D12 => rand::thread_rng().gen_range(1..=12),
            Dice::D20 => rand::thread_rng().gen_range(1..=20),
        }
    }
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
        };

        self.base_armor_class + ac_from_dex
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
}
