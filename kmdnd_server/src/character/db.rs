use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson;

use crate::campaign::CampaignId;
use crate::database::MongoCharacterStore;
use crate::error::Error;

use super::{Character, CharacterId, Position};

#[async_trait]
pub trait CharacterStore {
    async fn insert_character(&self, character: &Character) -> Result<(), Error>;

    async fn fetch_characters_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<Character>, Error>;

    async fn fetch_character_by_campaign_and_id(
        &self,
        campaign_id: CampaignId,
        character_id: CharacterId,
    ) -> Result<Option<Character>, Error>;

    async fn update_character_position(
        &self,
        mut character: Character,
        position: Option<Position>,
    ) -> Result<Character, Error>;

    async fn update_character_hit_points(
        &self,
        mut character: Character,
        hit_points: i32,
    ) -> Result<Character, Error>;
}

#[async_trait]
impl CharacterStore for MongoCharacterStore {
    #[tracing::instrument(skip(self))]
    async fn insert_character(&self, character: &Character) -> Result<(), Error> {
        self.insert_one(character, None).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_characters_by_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<Character>, Error> {
        let characters: Vec<Character> = self
            .find(bson::doc! { "owner.campaign_id": campaign_id }, None)
            .await?
            .try_collect()
            .await?;

        Ok(characters)
    }

    #[tracing::instrument(skip(self))]
    async fn fetch_character_by_campaign_and_id(
        &self,
        campaign_id: CampaignId,
        character_id: CharacterId,
    ) -> Result<Option<Character>, Error> {
        let character: Option<Character> = self
            .find_one(
                bson::doc! { "_id": character_id, "owner.campaign_id": campaign_id },
                None,
            )
            .await?;

        Ok(character)
    }

    #[tracing::instrument(skip(self))]
    async fn update_character_position(
        &self,
        mut character: Character,
        position: Option<Position>,
    ) -> Result<Character, Error> {
        let now = Utc::now();
        let old_modified_at = bson::DateTime::from_chrono(character.modified_at);
        let new_modified_at = bson::DateTime::from_chrono(now);
        let new_position = bson::to_document(&position)?;

        let result = self
            .update_one(
                bson::doc! { "_id": character.id, "modified_at": old_modified_at },
                bson::doc! { "$set": { "position": new_position, "modified_at": new_modified_at } },
                None,
            )
            .await?;

        if result.matched_count == 0 {
            return Err(Error::ConcurrentModificationDetected);
        }

        character.modified_at = now;
        character.position = position;

        Ok(character)
    }

    #[tracing::instrument(skip(self))]
    async fn update_character_hit_points(
        &self,
        mut character: Character,
        hit_points: i32,
    ) -> Result<Character, Error> {
        let now = Utc::now();
        let old_modified_at = bson::DateTime::from_chrono(character.modified_at);
        let new_modified_at = bson::DateTime::from_chrono(now);
        let new_hit_points = bson::to_bson(&hit_points)?;

        let result = self
            .update_one(
                bson::doc! { "_id": character.id, "modified_at": old_modified_at },
                bson::doc! { "$set": { "current_hit_points": new_hit_points, "modified_at": new_modified_at } },
                None,
            )
            .await?;

        if result.matched_count == 0 {
            return Err(Error::ConcurrentModificationDetected);
        }

        character.modified_at = now;
        character.current_hit_points = hit_points;

        Ok(character)
    }
}
