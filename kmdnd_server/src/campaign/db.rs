use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::database::CampaignStore;
use crate::error::Error;

use super::{Campaign, CampaignId};

const CAMPAIGNS: &str = "campaigns";

pub async fn initialize(_db: &Database) -> Result<(), Error> {
    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn insert_campaign(db: &CampaignStore, campaign: &Campaign) -> Result<(), Error> {
    db.insert_one(campaign, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_campaigns(db: &CampaignStore) -> Result<Vec<Campaign>, Error> {
    let campaigns: Vec<Campaign> = db.find(bson::doc! {}, None).await?.try_collect().await?;

    Ok(campaigns)
}

#[tracing::instrument(skip(db))]
pub async fn assert_campaign_exists(
    db: &CampaignStore,
    campaign_id: CampaignId,
) -> Result<Campaign, Error> {
    fetch_campaign_by_id(db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })
}

#[tracing::instrument(skip(db))]
pub async fn fetch_campaign_by_id(
    db: &CampaignStore,
    campaign_id: CampaignId,
) -> Result<Option<Campaign>, Error> {
    let campaign: Option<Campaign> = db.find_one(bson::doc! { "_id": campaign_id }, None).await?;

    Ok(campaign)
}
