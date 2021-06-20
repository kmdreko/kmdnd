use futures::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::encounter::EncounterId;
use crate::error::Error;

use super::Operation;

const OPERATIONS: &str = "operations";

pub async fn initialize(db: &Database) -> Result<(), Error> {
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

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn insert_operation(db: &Database, operation: &Operation) -> Result<(), Error> {
    let doc = bson::to_document(operation)?;
    db.collection(OPERATIONS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
        .collection(OPERATIONS)
        .find(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(operations)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_encounter(
    db: &Database,
    encounter_id: EncounterId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
        .collection(OPERATIONS)
        .find(bson::doc! { "encounter_id": encounter_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(operations)
}
