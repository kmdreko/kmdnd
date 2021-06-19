use futures::TryStreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::error::Error;

use super::{Encounter, EncounterId, EncounterState};

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
    encounter_id: EncounterId,
    state: EncounterState,
) -> Result<(), Error> {
    db.collection::<Encounter>(ENCOUNTERS)
        .update_one(
            bson::doc! { "_id": encounter_id },
            bson::doc! { "$set": { "state": bson::to_document(&state)? } },
            None,
        )
        .await?;

    Ok(())
}
