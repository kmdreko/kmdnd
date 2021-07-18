use chrono::Utc;

use crate::campaign::Campaign;
use crate::character::race::{Race, RacialTrait};
use crate::character::{
    Character, CharacterOwner, CharacterStats, EquipmentEntry, Language, Position, Proficiencies,
    ToolType,
};
use crate::database::Database;
use crate::encounter::{Encounter, EncounterId, EncounterState};
use crate::error::Error;
use crate::item::{
    Armor, ArmorType, DamageType, Dice, Item, ItemId, ItemType, Range, Weapon, WeaponProperty,
};
use crate::operation::{AbilityType, SkillType};

pub async fn seed(db: &dyn Database) -> Result<(), Error> {
    db.drop().await?;

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
        db.items().insert_item(item).await?;
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
        race: Race::HalfOrc,
        proficiencies: Proficiencies {
            armor: vec![ArmorType::Light, ArmorType::Medium, ArmorType::Shield],
            tool: vec![],
            saving_throws: vec![AbilityType::Strength, AbilityType::Constitution],
            skills: vec![SkillType::Athletics, SkillType::Intimidation],
        },
        racial_traits: vec![
            RacialTrait::AbilityScoreIncrease(vec![
                AbilityType::Strength,
                AbilityType::Strength,
                AbilityType::Constitution,
            ]),
            RacialTrait::Darkvision,
            RacialTrait::Menacing,
            RacialTrait::RelentlessEndurance,
            RacialTrait::SavageAttacks,
            RacialTrait::Languages(vec![Language::Common, Language::Orc]),
        ],
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
        race: Race::Gnome,
        proficiencies: Proficiencies {
            armor: vec![ArmorType::Light],
            tool: vec![ToolType::Lute, ToolType::Shawm, ToolType::PanFlute],
            saving_throws: vec![AbilityType::Dexterity, AbilityType::Charisma],
            skills: vec![SkillType::SleightOfHand, SkillType::Nature],
        },
        racial_traits: vec![
            RacialTrait::AbilityScoreIncrease(vec![
                AbilityType::Intelligence,
                AbilityType::Intelligence,
            ]),
            RacialTrait::Darkvision,
            RacialTrait::GnomeCunning,
            RacialTrait::Languages(vec![Language::Common, Language::Gnomish]),
        ],
    };

    character1.recalculate_stats(db).await?;
    character2.recalculate_stats(db).await?;

    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id: campaign.id,
        character_ids: vec![character1_id, character2_id],
        created_at: now,
        modified_at: now,
        state: EncounterState::Initiative,
    };

    db.campaigns().insert_campaign(&campaign).await?;
    db.characters().insert_character(&character1).await?;
    db.characters().insert_character(&character2).await?;
    db.encounters().insert_encounter(&encounter).await?;

    Ok(())
}
