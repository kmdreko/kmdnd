use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::error::Error;
use crate::{Campaign, CampaignId, Character, CharacterId};

const CAMPAIGNS: &str = "campaigns";
const CHARACTERS: &str = "characters";

#[tracing::instrument(skip(db))]
pub async fn insert_campaign(db: &Database, campaign: &Campaign) -> Result<(), Error> {
    let doc = bson::to_document(campaign)?;
    db.collection(CAMPAIGNS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_campaigns(db: &Database) -> Result<Vec<Campaign>, Error> {
    let campaigns: Vec<Campaign> = db
        .collection(CAMPAIGNS)
        .find(bson::doc! {}, None)
        .await?
        .try_collect()
        .await?;

    Ok(campaigns)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_campaign_by_id(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Option<Campaign>, Error> {
    let campaign: Option<Campaign> = db
        .collection(CAMPAIGNS)
        .find_one(bson::doc! { "_id": campaign_id }, None)
        .await?;

    Ok(campaign)
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
