use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson, Database};

use crate::database::MongoCampaignStore;
use crate::error::Error;

use super::{Campaign, CampaignId};

const CAMPAIGNS: &str = "campaigns";

pub async fn initialize(_db: &Database) -> Result<(), Error> {
    Ok(())
}

#[async_trait]
pub trait CampaignStore {
    async fn insert_campaign(&self, campaign: &Campaign) -> Result<(), Error>;

    async fn fetch_campaigns(&self) -> Result<Vec<Campaign>, Error>;

    async fn assert_campaign_exists(&self, campaign_id: CampaignId) -> Result<Campaign, Error>;

    async fn fetch_campaign_by_id(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<Campaign>, Error>;
}

#[async_trait]
impl CampaignStore for MongoCampaignStore {
    #[tracing::instrument(skip(self))]
    async fn insert_campaign(&self, campaign: &Campaign) -> Result<(), Error> {
        self.insert_one(campaign, None).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_campaigns(&self) -> Result<Vec<Campaign>, Error> {
        let campaigns: Vec<Campaign> = self.find(bson::doc! {}, None).await?.try_collect().await?;

        Ok(campaigns)
    }

    #[tracing::instrument(skip(self))]
    async fn assert_campaign_exists(&self, campaign_id: CampaignId) -> Result<Campaign, Error> {
        self.fetch_campaign_by_id(campaign_id)
            .await?
            .ok_or(Error::CampaignDoesNotExist { campaign_id })
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_campaign_by_id(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<Campaign>, Error> {
        let campaign: Option<Campaign> = self
            .find_one(bson::doc! { "_id": campaign_id }, None)
            .await?;

        Ok(campaign)
    }
}
