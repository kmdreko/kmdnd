use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::campaign::CampaignId;
use crate::error::Error;

use super::{Character, CharacterId, Position};

const CHARACTERS: &str = "characters";

pub async fn initialize(db: &Database) -> Result<(), Error> {
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

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn insert_character(db: &Database, character: &Character) -> Result<(), Error> {
    let doc = bson::to_document(character)?;
    db.collection(CHARACTERS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_characters_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Vec<Character>, Error> {
    let characters: Vec<Character> = db
        .collection(CHARACTERS)
        .find(bson::doc! { "owner.campaign_id": campaign_id }, None)
        .await?
        .try_collect()
        .await?;

    Ok(characters)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_character_by_campaign_and_id(
    db: &Database,
    campaign_id: CampaignId,
    character_id: CharacterId,
) -> Result<Option<Character>, Error> {
    let character: Option<Character> = db
        .collection(CHARACTERS)
        .find_one(
            bson::doc! { "_id": character_id, "owner.campaign_id": campaign_id },
            None,
        )
        .await?;

    Ok(character)
}

#[tracing::instrument(skip(db))]
pub async fn update_character_position(
    db: &Database,
    character: &Character,
    position: Option<Position>,
) -> Result<(), Error> {
    let position = bson::to_document(&position)?;
    let old_modified_at = bson::DateTime::from_chrono(character.modified_at);
    let new_modified_at = bson::DateTime::from_chrono(Utc::now());

    let result = db
        .collection::<Character>(CHARACTERS)
        .update_one(
            bson::doc! { "_id": character.id, "modified_at": old_modified_at },
            bson::doc! { "$set": { "position": position, "modified_at": new_modified_at } },
            None,
        )
        .await?;

    if result.matched_count == 0 {
        return Err(Error::ConcurrentModificationDetected);
    }

    Ok(())
}
