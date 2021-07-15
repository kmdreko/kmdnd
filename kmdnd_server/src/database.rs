use mongodb::{Collection, Database};

use crate::campaign::db::CampaignStore;
use crate::campaign::Campaign;
use crate::character::db::CharacterStore;
use crate::character::Character;
use crate::encounter::db::EncounterStore;
use crate::encounter::Encounter;
use crate::error::Error;
use crate::item::db::ItemStore;
use crate::item::Item;
use crate::operation::db::OperationStore;
use crate::operation::Operation;

pub type MongoCampaignStore = Collection<Campaign>;
pub type MongoCharacterStore = Collection<Character>;
pub type MongoEncounterStore = Collection<Encounter>;
pub type MongoItemStore = Collection<Item>;
pub type MongoOperationStore = Collection<Operation>;

#[derive(Debug, Clone)]
pub struct MongoDatabase {
    campaigns: Collection<Campaign>,
    characters: Collection<Character>,
    encounters: Collection<Encounter>,
    items: Collection<Item>,
    operations: Collection<Operation>,
    db: Database,
}

impl MongoDatabase {
    pub fn new(db: Database) -> MongoDatabase {
        MongoDatabase {
            campaigns: db.collection("campaigns"),
            characters: db.collection("characters"),
            encounters: db.collection("encounters"),
            items: db.collection("items"),
            operations: db.collection("operations"),
            db: db,
        }
    }

    pub fn campaigns(&self) -> &dyn CampaignStore {
        &self.campaigns
    }

    pub fn characters(&self) -> &dyn CharacterStore {
        &self.characters
    }

    pub fn encounters(&self) -> &dyn EncounterStore {
        &self.encounters
    }

    pub fn items(&self) -> &dyn ItemStore {
        &self.items
    }

    pub fn operations(&self) -> &dyn OperationStore {
        &self.operations
    }

    pub async fn drop(&self) -> Result<(), Error> {
        self.db.drop(None).await?;
        Ok(())
    }
}
