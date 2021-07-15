// pub trait CampaignStore {

// }

// pub trait Database {
//     fn campaigns(&self) -> &dyn CampaignStore;
// }

use mongodb::{Collection, Database};

use crate::campaign::Campaign;
use crate::character::Character;
use crate::encounter::Encounter;
use crate::error::Error;
use crate::item::Item;
use crate::operation::Operation;

pub type CampaignStore = Collection<Campaign>;
pub type CharacterStore = Collection<Character>;
pub type EncounterStore = Collection<Encounter>;
pub type ItemStore = Collection<Item>;
pub type OperationStore = Collection<Operation>;

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

    pub fn campaigns(&self) -> &CampaignStore {
        &self.campaigns
    }

    pub fn characters(&self) -> &CharacterStore {
        &self.characters
    }

    pub fn encounters(&self) -> &EncounterStore {
        &self.encounters
    }

    pub fn items(&self) -> &ItemStore {
        &self.items
    }

    pub fn operations(&self) -> &OperationStore {
        &self.operations
    }

    pub async fn drop(&self) -> Result<(), Error> {
        self.db.drop(None).await?;
        Ok(())
    }
}
