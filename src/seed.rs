use chrono::Utc;
use mongodb::Database;

use crate::campaign::{self, Campaign};
use crate::character::{self, Character, CharacterOwner, CharacterStats};
use crate::encounter::{self, Encounter, EncounterId, EncounterState};

pub async fn seed(db: &Database) {
    let now = Utc::now();
    let campaign_id = "CPN-16E77539-8873-4C8A-BCA3-2036010474AD".parse().unwrap();
    let campaign = Campaign {
        id: campaign_id,
        name: "The Green Bean Bunch".to_string(),
        created_at: now,
        modified_at: now,
    };

    let character1_id = "CHR-33957EB6-0EE7-487F-A087-E55C335BD63C".parse().unwrap();
    let character1 = Character {
        id: character1_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "Mr. Understanding".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
    };

    let character2_id = "CHR-DE3168FD-2730-47A2-BFE0-E53C79DD57A0".parse().unwrap();
    let character2 = Character {
        id: character2_id,
        owner: CharacterOwner::Campaign(campaign_id),
        name: "The Chi Bee".to_string(),
        created_at: now,
        modified_at: now,
        stats: CharacterStats::default(),
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
}
