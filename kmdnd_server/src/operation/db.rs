use chrono::Utc;
use futures::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::database::OperationStore;
use crate::encounter::{EncounterId, Round};
use crate::error::Error;

use super::{Interaction, Legality, Operation, OperationId};

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
pub async fn insert_operation(db: &OperationStore, operation: &Operation) -> Result<(), Error> {
    db.insert_one(operation, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operation_by_id(
    db: &OperationStore,
    operation_id: OperationId,
) -> Result<Option<Operation>, Error> {
    let operation = db
        .find_one(bson::doc! { "_id": operation_id }, None)
        .await?;

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_campaign(
    db: &OperationStore,
    campaign_id: CampaignId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
        .find(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(operations)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_encounter(
    db: &OperationStore,
    encounter_id: EncounterId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
        .find(bson::doc! { "encounter_id": encounter_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(operations)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_operations_by_turn(
    db: &OperationStore,
    encounter_id: EncounterId,
    round: Round,
    character_id: CharacterId,
) -> Result<Vec<Operation>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let operations: Vec<Operation> = db
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
    db: &OperationStore,
    mut operation: Operation,
    interaction_index: usize,
    interaction_result: i32,
) -> Result<Operation, Error> {
    let now = Utc::now();
    let old_modified_at = bson::DateTime::from_chrono(operation.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(now);
    let result_path = format!("interactions.{}.result", interaction_index);

    let result = db
        .update_one(
            bson::doc! { "_id": operation.id, "modified_at": old_modified_at },
            bson::doc! { "$set": { result_path: interaction_result, "modified_at": new_modified_at } },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    operation.modified_at = now;
    operation.interactions[interaction_index].result = Some(interaction_result);

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn update_operation_push_interactions(
    db: &OperationStore,
    mut operation: Operation,
    interactions: Vec<Interaction>,
) -> Result<Operation, Error> {
    let now = Utc::now();
    let old_modified_at = bson::DateTime::from_chrono(operation.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(now);
    let new_interactions = bson::to_bson(&interactions)?;

    let result = db
        .update_one(
            bson::doc! { "_id": operation.id, "modified_at": old_modified_at },
            bson::doc! {
                "$push": { "interactions": { "$each": new_interactions } },
                "$set": { "modified_at": new_modified_at }
            },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    operation.modified_at = now;
    operation.interactions.extend(interactions);

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn update_operation_legality(
    db: &OperationStore,
    mut operation: Operation,
    legality: Legality,
) -> Result<Operation, Error> {
    let now = Utc::now();
    let old_modified_at = bson::DateTime::from_chrono(operation.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(now);
    let new_legality = bson::to_bson(&legality)?;

    let result = db
        .update_one(
            bson::doc! { "_id": operation.id, "modified_at": old_modified_at },
            bson::doc! { "$set": { "legality": new_legality, "modified_at": new_modified_at } },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    operation.modified_at = now;
    operation.legality = legality;

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn delete_operation(db: &OperationStore, operation_id: OperationId) -> Result<(), Error> {
    db.delete_one(bson::doc! { "_id": operation_id }, None)
        .await?;

    Ok(())
}
