use chrono::Utc;
use mongodb::Database;

use crate::campaign::{self, Campaign};
use crate::character::{self, Character, CharacterOwner, CharacterStats, EquipmentEntry, Position};
use crate::encounter::{self, Encounter, EncounterId, EncounterState};
use crate::error::Error;
use crate::item::{
    self, Armor, ArmorType, DamageType, Dice, Item, ItemId, ItemType, Range, Weapon, WeaponProperty,
};

pub async fn seed(db: &Database) -> Result<(), Error> {
    db.drop(None).await?;

    let campaign_id = "CPN-16E77539-8873-4C8A-BCA3-2036010474AD".parse().unwrap();
    let item1_id = "ITM-5EA81D0A-9788-4B8A-82D9-1A0D636B53CE".parse().unwrap();
    let item2_id = "ITM-5C903E93-2524-4876-B4C8-816B98D0C77B".parse().unwrap();
    let character1_id = "CHR-33957EB6-0EE7-487F-A087-E55C335BD63C".parse().unwrap();
    let character2_id = "CHR-DE3168FD-2730-47A2-BFE0-E53C79DD57A0".parse().unwrap();

    let now = Utc::now();
    let campaign = Campaign {
        id: campaign_id,
        name: "The Green Bean Bunch".to_string(),
        created_at: now,
        modified_at: now,
    };

    let items = vec![
        Item {
            id: item1_id,
            name: "Club".to_string(),
            value: 10,
            weight: 2,
            item_type: ItemType::Weapon(Weapon {
                damage_amount: Dice::D4,
                damage_type: DamageType::Bludgeoning,
                properties: vec![WeaponProperty::Light],
            }),
        },
        Item {
            id: item2_id,
            name: "Shortbow".to_string(),
            value: 2500,
            weight: 2,
            item_type: ItemType::Weapon(Weapon {
                damage_amount: Dice::D6,
                damage_type: DamageType::Piercing,
                properties: vec![
                    WeaponProperty::Ammunition(Range {
                        normal: 80,
                        long: 320,
                    }),
                    WeaponProperty::TwoHanded,
                ],
            }),
        },
        Item {
            id: ItemId::new(),
            name: "Scale mail".to_string(),
            value: 5000,
            weight: 45,
            item_type: ItemType::Armor(Armor {
                base_armor_class: 4,
                armor_type: ArmorType::Medium,
                strength_requirement: None,
                stealth_disadvantage: true,
            }),
        },
        Item {
            id: ItemId::new(),
            name: "Studded leather".to_string(),
            value: 4500,
            weight: 13,
            item_type: ItemType::Armor(Armor {
                base_armor_class: 2,
                armor_type: ArmorType::Light,
                strength_requirement: None,
                stealth_disadvantage: false,
            }),
        },
    ];

    for item in &items {
        item::db::insert_item(db, item).await?;
    }

    let mut character1 = Character {
        id: character1_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "Mr. Understanding".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
        equipment: vec![
            EquipmentEntry {
                equiped: true,
                quantity: 1,
                item_id: items[0].id,
            },
            EquipmentEntry {
                equiped: true,
                quantity: 1,
                item_id: items[2].id,
            },
        ],
        position: Some(Position {
            x: 5.0,
            y: 0.0,
            z: 0.0,
        }),
        current_hit_points: 10,
        maximum_hit_points: 10,
    };

    let mut character2 = Character {
        id: character2_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "The Chi Bee".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
        equipment: vec![
            EquipmentEntry {
                equiped: true,
                quantity: 1,
                item_id: items[1].id,
            },
            EquipmentEntry {
                equiped: true,
                quantity: 1,
                item_id: items[3].id,
            },
        ],
        position: Some(Position {
            x: -5.0,
            y: 0.0,
            z: 0.0,
        }),
        current_hit_points: 10,
        maximum_hit_points: 10,
    };

    character1.recalculate_stats(db).await?;
    character2.recalculate_stats(db).await?;

    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id: campaign_id,
        character_ids: vec![character1_id, character2_id],
        created_at: now,
        modified_at: now,
        state: EncounterState::Initiative,
    };

    campaign::db::insert_campaign(db, &campaign).await?;
    character::db::insert_character(db, &character1).await?;
    character::db::insert_character(db, &character2).await?;
    encounter::db::insert_encounter(db, &encounter).await?;

    Ok(())
}
