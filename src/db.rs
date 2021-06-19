use futures::TryStreamExt;
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{bson, Database};

use crate::campaign::{Campaign, CampaignId};
use crate::character::{Character, CharacterId};
use crate::encounter::{Encounter, EncounterId, EncounterState};
use crate::error::Error;

const CAMPAIGNS: &str = "campaigns";
const CHARACTERS: &str = "characters";
const ENCOUNTERS: &str = "encounters";

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

#[tracing::instrument(skip(db))]
pub async fn insert_encounter(db: &Database, encounter: &Encounter) -> Result<(), Error> {
    let doc = bson::to_document(encounter)?;
    db.collection(ENCOUNTERS).insert_one(doc, None).await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn fetch_encounters_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Vec<Encounter>, Error> {
    let options = FindOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let encounters: Vec<Encounter> = db
        .collection(ENCOUNTERS)
        .find(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .try_collect()
        .await?;

    Ok(encounters)
}

#[tracing::instrument(skip(db))]
pub async fn fetch_current_encounter_by_campaign(
    db: &Database,
    campaign_id: CampaignId,
) -> Result<Option<Encounter>, Error> {
    let options = FindOneOptions::builder()
        .sort(bson::doc! { "created_at": -1 })
        .build();

    let encounter: Option<Encounter> = db
        .collection(ENCOUNTERS)
        .find_one(bson::doc! { "campaign_id": campaign_id }, options)
        .await?
        .filter(|e: &Encounter| e.state != EncounterState::Finished);

    Ok(encounter)
}

#[tracing::instrument(skip(db))]
pub async fn update_encounter_state(
    db: &Database,
    encounter_id: EncounterId,
    state: EncounterState,
) -> Result<(), Error> {
    db.collection::<Encounter>(ENCOUNTERS)
        .update_one(
            bson::doc! { "_id": encounter_id },
            bson::doc! { "$set": { "state": bson::to_document(&state)? } },
            None,
        )
        .await?;

    Ok(())
}
