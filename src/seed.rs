use chrono::Utc;
use mongodb::Database;

use crate::campaign::{self, Campaign};
use crate::character::{self, Character, CharacterOwner, CharacterStats, ItemWithQuantity};
use crate::encounter::{self, Encounter, EncounterId, EncounterState};
use crate::item::{self, DamageType, Dice, Item, ItemType, Weapon, WeaponProperty};

pub async fn seed(db: &Database) {
    let now = Utc::now();
    let campaign_id = "CPN-16E77539-8873-4C8A-BCA3-2036010474AD".parse().unwrap();
    let campaign = Campaign {
        id: campaign_id,
        name: "The Green Bean Bunch".to_string(),
        created_at: now,
        modified_at: now,
    };

    let item1_id = "ITM-5EA81D0A-9788-4B8A-82D9-1A0D636B53CE".parse().unwrap();
    let item2_id = "ITM-5C903E93-2524-4876-B4C8-816B98D0C77B".parse().unwrap();
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
                    WeaponProperty::Ammunition {
                        normal_range: 80,
                        long_range: 320,
                    },
                    WeaponProperty::TwoHanded,
                ],
            }),
        },
    ];

    let character1_id = "CHR-33957EB6-0EE7-487F-A087-E55C335BD63C".parse().unwrap();
    let character1 = Character {
        id: character1_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "Mr. Understanding".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
        equipment: vec![ItemWithQuantity {
            quantity: 1,
            item_id: items[0].id,
        }],
    };

    let character2_id = "CHR-DE3168FD-2730-47A2-BFE0-E53C79DD57A0".parse().unwrap();
    let character2 = Character {
        id: character2_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "The Chi Bee".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
        equipment: vec![ItemWithQuantity {
            quantity: 1,
            item_id: items[1].id,
        }],
    };

    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id: campaign_id,
        character_ids: vec![character1_id, character2_id],
        created_at: now,
        modified_at: now,
        state: EncounterState::Initiative,
    };

    db.drop(None).await.unwrap();

    campaign::db::insert_campaign(db, &campaign).await.unwrap();
    character::db::insert_character(db, &character1)
        .await
        .unwrap();
    character::db::insert_character(db, &character2)
        .await
        .unwrap();
    encounter::db::insert_encounter(db, &encounter)
        .await
        .unwrap();

    for item in items {
        item::db::insert_item(db, &item).await.unwrap();
    }
}
