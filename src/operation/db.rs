use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::encounter::{EncounterId, Round};
use crate::error::Error;

use super::{Interaction, Operation, OperationId};

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
pub async fn fetch_operation_by_id(
    db: &Database,
    operation_id: OperationId,
) -> Result<Option<Operation>, Error> {
    let operation = db
        .collection(OPERATIONS)
        .find_one(bson::doc! { "_id": operation_id }, None)
        .await?;

    Ok(operation)
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

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_turn(
    db: &Database,
    encounter_id: EncounterId,
    round: Round,
    character_id: CharacterId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
        .collection(OPERATIONS)
        .find(
            bson::doc! {
                "encounter_id": encounter_id,
                "encounter_state.type": "TURN",
                "encounter_state.round": round,
                "encounter_state.character_id": character_id,
            },
            options,
        )
        .await?
        .try_collect()
        .await?;

    Ok(operations)
}

#[tracing::instrument(skip(db))]
pub async fn update_operation_interaction_result(
    db: &Database,
    operation: &Operation,
    interaction_index: usize,
    result: i32,
) -> Result<DateTime<Utc>, Error> {
    let now = Utc::now();
    let old_modified_at = bson::DateTime::from_chrono(operation.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(now);
    let result_path = format!("interactions.{}.result", interaction_index);

    let result = db
        .collection::<Operation>(OPERATIONS)
        .update_one(
            bson::doc! { "_id": operation.id, "modified_at": old_modified_at },
            bson::doc! { "$set": { result_path: result, "modified_at": new_modified_at } },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    Ok(now)
}

#[tracing::instrument(skip(db))]
pub async fn update_operation_push_interactions(
    db: &Database,
    operation: &Operation,
    interactions: &[Interaction],
) -> Result<(), Error> {
    let interactions = bson::to_bson(interactions)?;
    let old_modified_at = bson::DateTime::from_chrono(operation.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(Utc::now());

    let result = db
        .collection::<Operation>(OPERATIONS)
        .update_one(
            bson::doc! { "_id": operation.id, "modified_at": old_modified_at },
            bson::doc! {
                "$push": { "interactions": { "$each": interactions } },
                "$set": { "modified_at": new_modified_at }
            },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    Ok(())
}
