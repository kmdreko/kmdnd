use chrono::Utc;

use crate::database::Database;
use crate::error::Error;

use super::{Campaign, CampaignId};

#[tracing::instrument(skip(db))]
pub async fn create_campaign(db: &dyn Database, name: String) -> Result<Campaign, Error> {
    let now = Utc::now();
    let campaign = Campaign {
        id: CampaignId::new(),
        name: name,
        created_at: now,
        modified_at: now,
    };

    db.campaigns().insert_campaign(&campaign).await?;

    Ok(campaign)
}

#[tracing::instrument(skip(db))]
pub async fn get_campaigns(db: &dyn Database) -> Result<Vec<Campaign>, Error> {
    let campaigns = db.campaigns().fetch_campaigns().await?;

    Ok(campaigns)
}

#[tracing::instrument(skip(db))]
pub async fn get_campaign_by_id(
    db: &dyn Database,
    campaign_id: CampaignId,
) -> Result<Campaign, Error> {
    let campaign = db
        .campaigns()
        .fetch_campaign_by_id(campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    Ok(campaign)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::test::MockDatabase;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn can_create_campaign() {
        let mut db = MockDatabase::new();
        let called_insert = Arc::new(Mutex::new(false));
        let called_insert_clone = Arc::clone(&called_insert);
        db.campaigns.on_insert_campaign = Box::new(move |campaign| {
            *called_insert_clone.lock().unwrap() = true;
            assert_eq!(campaign.name, "Blue Man Group".to_string());
            assert_eq!(campaign.created_at, campaign.modified_at);
            Ok(())
        });

        let campaign = create_campaign(&db, "Blue Man Group".into()).await.unwrap();

        assert_eq!(campaign.name, "Blue Man Group".to_string());
        assert_eq!(campaign.created_at, campaign.modified_at);
        assert!(
            *called_insert.lock().unwrap(),
            "db.insert_campaign was not called"
        );
    }

    #[tokio::test]
    async fn get_campaign_by_id_returns_campaign() {
        let mut db = MockDatabase::new();
        let test_campaign_id = CampaignId::new();
        let called_get_by_id = Arc::new(Mutex::new(false));
        let called_get_by_id_clone = Arc::clone(&called_get_by_id);
        db.campaigns.on_fetch_campaign_by_id = Box::new(move |campaign_id| {
            *called_get_by_id_clone.lock().unwrap() = true;
            assert_eq!(campaign_id, test_campaign_id);
            let now = Utc::now();
            Ok(Some(Campaign {
                id: campaign_id,
                name: "Blue Man Group".to_string(),
                created_at: now,
                modified_at: now,
            }))
        });

        let campaign = get_campaign_by_id(&db, test_campaign_id).await.unwrap();

        assert_eq!(campaign.name, "Blue Man Group".to_string());
        assert_eq!(campaign.created_at, campaign.modified_at);
        assert!(
            *called_get_by_id.lock().unwrap(),
            "db.fetch_campaign_by_id was not called"
        );
    }

    #[tokio::test]
    async fn get_campaign_by_id_returns_error_if_doesnt_exist() {
        let mut db = MockDatabase::new();
        let test_campaign_id = CampaignId::new();
        let called_get_by_id = Arc::new(Mutex::new(false));
        let called_get_by_id_clone = Arc::clone(&called_get_by_id);
        db.campaigns.on_fetch_campaign_by_id = Box::new(move |campaign_id| {
            *called_get_by_id_clone.lock().unwrap() = true;
            assert_eq!(campaign_id, test_campaign_id);
            Ok(None)
        });

        let campaign_result = get_campaign_by_id(&db, test_campaign_id).await;

        assert_eq!(
            campaign_result.unwrap_err(),
            Error::CampaignDoesNotExist {
                campaign_id: test_campaign_id
            }
        );
        assert!(
            *called_get_by_id.lock().unwrap(),
            "db.fetch_campaign_by_id was not called"
        );
    }
}
