use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::error::Error;

use super::{Campaign, CampaignId};

const CAMPAIGNS: &str = "campaigns";

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
