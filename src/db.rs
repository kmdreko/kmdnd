use std::io::{Error, ErrorKind};

use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::{Campaign, CampaignId, Character};

const CAMPAIGNS: &str = "campaigns";
const CHARACTERS: &str = "characters";

#[tracing::instrument(skip(db))]
pub async fn insert_campaign(db: &Database, campaign: &Campaign) -> Result<(), Error> {
    let doc = bson::to_document(campaign).map_err(|err| Error::new(ErrorKind::Other, err))?;
    db.collection(CAMPAIGNS)
        .insert_one(doc, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_campaigns(db: &Database) -> Result<Vec<Campaign>, Error> {
    let campaigns: Vec<Campaign> = db
        .collection(CAMPAIGNS)
        .find(bson::doc! {}, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .try_collect()
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

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
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(campaign)
}

#[tracing::instrument(skip(db))]
pub async fn insert_character(db: &Database, character: &Character) -> Result<(), Error> {
    let doc = bson::to_document(character).map_err(|err| Error::new(ErrorKind::Other, err))?;
    db.collection(CHARACTERS)
        .insert_one(doc, None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

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
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?
        .try_collect()
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    Ok(characters)
}
