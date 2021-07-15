use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::database::MongoEncounterStore;
use crate::error::Error;

use super::{Encounter, EncounterState};

const ENCOUNTERS: &str = "encounters";

pub async fn initialize(db: &Database) -> Result<(), Error> {
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

    Ok(())
}

#[async_trait]
pub trait EncounterStore {
    async fn insert_encounter(&self, encounter: &Encounter) -> Result<(), Error>;

    async fn fetch_encounters_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<Encounter>, Error>;

    async fn assert_current_encounter_exists(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Encounter, Error>;

    async fn fetch_current_encounter_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<Encounter>, Error>;

    async fn update_encounter_state(
        &self,
        mut encounter: Encounter,
        state: EncounterState,
    ) -> Result<Encounter, Error>;

    async fn update_encounter_state_and_characters(
        &self,
        mut encounter: Encounter,
        state: EncounterState,
        character_ids: Vec<CharacterId>,
    ) -> Result<Encounter, Error>;
}

#[async_trait]
impl EncounterStore for MongoEncounterStore {
    #[tracing::instrument(skip(self))]
    async fn insert_encounter(&self, encounter: &Encounter) -> Result<(), Error> {
        self.insert_one(encounter, None).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_encounters_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<Encounter>, Error> {
        let options = FindOptions::builder()
            .sort(bson::doc! { "created_at": -1 })
            .build();

        let encounters: Vec<Encounter> = self
            .find(bson::doc! { "campaign_id": campaign_id }, options)
            .await?
            .try_collect()
            .await?;

        Ok(encounters)
    }

    #[tracing::instrument(skip(self))]
    async fn assert_current_encounter_exists(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Encounter, Error> {
        self.fetch_current_encounter_by_campaign(campaign_id)
            .await?
            .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_current_encounter_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<Encounter>, Error> {
        let options = FindOneOptions::builder()
            .sort(bson::doc! { "created_at": -1 })
            .build();

        let encounter: Option<Encounter> = self
            .find_one(bson::doc! { "campaign_id": campaign_id }, options)
            .await?
            .filter(|e: &Encounter| e.state != EncounterState::Finished);

        Ok(encounter)
    }

    #[tracing::instrument(skip(self))]
    async fn update_encounter_state(
        &self,
        mut encounter: Encounter,
        state: EncounterState,
    ) -> Result<Encounter, Error> {
        let now = Utc::now();
        let new_state = bson::to_document(&state)?;
        let old_modified_at = bson::DateTime::from_chrono(encounter.modified_at);
        let new_modified_at = bson::DateTime::from_chrono(now);

        let result = self
            .update_one(
                bson::doc! { "_id": encounter.id, "modified_at": old_modified_at },
                bson::doc! { "$set": { "state": new_state, "modified_at": new_modified_at } },
                None,
            )
            .await?;

        if result.matched_count == 0 {
            return Err(Error::ConcurrentModificationDetected);
        }

        encounter.modified_at = now;
        encounter.state = state;

        Ok(encounter)
    }

    #[tracing::instrument(skip(self))]
    async fn update_encounter_state_and_characters(
        &self,
        mut encounter: Encounter,
        state: EncounterState,
        character_ids: Vec<CharacterId>,
    ) -> Result<Encounter, Error> {
        let now = Utc::now();
        let old_modified_at = bson::DateTime::from_chrono(encounter.modified_at);
        let new_modified_at = bson::DateTime::from_chrono(now);
        let new_state = bson::to_document(&state)?;
        let new_character_ids = bson::to_bson(&character_ids)?;

        let result = self
            .update_one(
                bson::doc! { "_id": encounter.id, "modified_at": old_modified_at },
                bson::doc! { "$set": {
                    "state": new_state,
                    "character_ids": new_character_ids,
                    "modified_at": new_modified_at
                } },
                None,
            )
            .await?;

        if result.matched_count == 0 {
            return Err(Error::ConcurrentModificationDetected);
        }

        encounter.modified_at = now;
        encounter.state = state;
        encounter.character_ids = character_ids;

        Ok(encounter)
    }
}
