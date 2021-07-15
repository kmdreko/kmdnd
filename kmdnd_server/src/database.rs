use async_trait::async_trait;
use mongodb::{bson, Collection};

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

#[async_trait]
pub trait Database {
    fn campaigns(&self) -> &dyn CampaignStore;
    fn characters(&self) -> &dyn CharacterStore;
    fn encounters(&self) -> &dyn EncounterStore;
    fn items(&self) -> &dyn ItemStore;
    fn operations(&self) -> &dyn OperationStore;

    async fn drop(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct MongoDatabase {
    campaigns: Collection<Campaign>,
    characters: Collection<Character>,
    encounters: Collection<Encounter>,
    items: Collection<Item>,
    operations: Collection<Operation>,
    db: mongodb::Database,
}

impl MongoDatabase {
    pub async fn initialize(db: mongodb::Database) -> Result<MongoDatabase, Error> {
        Ok(MongoDatabase {
            campaigns: initialize_campaigns(&db).await?,
            characters: initialize_characters(&db).await?,
            encounters: initialize_encounters(&db).await?,
            items: initialize_items(&db).await?,
            operations: initialize_operations(&db).await?,
            db: db,
        })
    }
}

#[async_trait]
impl Database for MongoDatabase {
    fn campaigns(&self) -> &dyn CampaignStore {
        &self.campaigns
    }

    fn characters(&self) -> &dyn CharacterStore {
        &self.characters
    }

    fn encounters(&self) -> &dyn EncounterStore {
        &self.encounters
    }

    fn items(&self) -> &dyn ItemStore {
        &self.items
    }

    fn operations(&self) -> &dyn OperationStore {
        &self.operations
    }

    async fn drop(&self) -> Result<(), Error> {
        self.db.drop(None).await?;
        Ok(())
    }
}

const CAMPAIGNS: &str = "campaigns";
const CHARACTERS: &str = "characters";
const ENCOUNTERS: &str = "encounters";
const ITEMS: &str = "items";
const OPERATIONS: &str = "operations";

pub async fn initialize_campaigns(db: &mongodb::Database) -> Result<MongoCampaignStore, Error> {
    Ok(db.collection(CAMPAIGNS))
}

pub async fn initialize_characters(db: &mongodb::Database) -> Result<MongoCharacterStore, Error> {
    db.run_command(
        bson::doc! {
            "createIndexes": CHARACTERS,
            "indexes": [
                { "key": { "owner.campaign_id": 1 }, "name": "by_campaign_id" },
                { "key": { "owner.user_id": 1 }, "name": "by_user_id" },
            ]
        },
        None,
    )
    .await?;

    Ok(db.collection(CHARACTERS))
}

pub async fn initialize_encounters(db: &mongodb::Database) -> Result<MongoEncounterStore, Error> {
    db.run_command(
        bson::doc! {
            "createIndexes": ENCOUNTERS,
            "indexes": [
                { "key": { "campaign_id": 1, "created_at": 1 }, "name": "by_campaign_id" },
            ]
        },
        None,
    )
    .await?;

    Ok(db.collection(ENCOUNTERS))
}

pub async fn initialize_items(db: &mongodb::Database) -> Result<MongoItemStore, Error> {
    Ok(db.collection(ITEMS))
}

pub async fn initialize_operations(db: &mongodb::Database) -> Result<MongoOperationStore, Error> {
    db.run_command(
        bson::doc! {
            "createIndexes": OPERATIONS,
            "indexes": [
                { "key": { "campaign_id": 1, "created_at": 1 }, "name": "by_campaign_id" },
                { "key": { "encounter_id": 1, "created_at": 1 }, "name": "by_encounter_id" },
            ]
        },
        None,
    )
    .await?;

    Ok(db.collection(OPERATIONS))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::campaign::db::MockCampaignStore;

    pub struct MockDatabase {
        pub campaigns: MockCampaignStore,
    }

    impl MockDatabase {
        pub fn new() -> MockDatabase {
            MockDatabase {
                campaigns: MockCampaignStore::new(),
            }
        }
    }

    #[async_trait]
    impl Database for MockDatabase {
        fn campaigns(&self) -> &dyn CampaignStore {
            &self.campaigns
        }

        fn characters(&self) -> &dyn CharacterStore {
            unimplemented!("MockDatabase::characters")
        }

        fn encounters(&self) -> &dyn EncounterStore {
            unimplemented!("MockDatabase::encounters")
        }

        fn items(&self) -> &dyn ItemStore {
            unimplemented!("MockDatabase::items")
        }

        fn operations(&self) -> &dyn OperationStore {
            unimplemented!("MockDatabase::operations")
        }

        async fn drop(&self) -> Result<(), Error> {
            unimplemented!("MockDatabase::drop")
        }
    }
}
