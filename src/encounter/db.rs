use chrono::Utc;
use futures::TryStreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::error::Error;

use super::{Encounter, EncounterState};

const ENCOUNTERS: &str = "encounters";

#[tracing::instrument(skip(db))]
pub async fn insert_encounter(db: &Database, encounter: &Encounter) -> Result<(), Error> {
    let doc = bson::to_document(encounter)?;
    db.collection(ENCOUNTERS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_encounters_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Vec<Encounter>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let encounters: Vec<Encounter> = db
        .collection(ENCOUNTERS)
        .find(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(encounters)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_current_encounter_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Option<Encounter>, Error> {
    let options = FindOneOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let encounter: Option<Encounter> = db
        .collection(ENCOUNTERS)
        .find_one(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .filter(|e: &Encounter| e.state != EncounterState::Finished);

    Ok(encounter)
}

#[tracing::instrument(skip(db))]
pub async fn update_encounter_state(
    db: &Database,
    encounter: &Encounter,
    state: EncounterState,
) -> Result<(), Error> {
    let state = bson::to_document(&state)?;
    let old_modified_at = bson::DateTime::from_chrono(encounter.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(Utc::now());

    let result = db
        .collection::<Encounter>(ENCOUNTERS)
        .update_one(
            bson::doc! { "_id": encounter.id, "modified_at": old_modified_at },
            bson::doc! { "$set": { "state": state, "modified_at": new_modified_at } },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    Ok(())
}