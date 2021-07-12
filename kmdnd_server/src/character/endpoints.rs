use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryStreamExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::race::Race;
use crate::character::Proficiencies;
use crate::error::Error;
use crate::item::{self, ItemBody};
use crate::operation::{AbilityType, RollType};

use super::{db, Character, CharacterId, CharacterOwner, CharacterStats, Position};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateCharacterBody {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CharacterBody {
    pub id: CharacterId,
    pub owner: CharacterOwner,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub stats: CharacterStats,
    pub equipment: Vec<ItemWithQuantityBody>,
    pub position: Option<Position>,
    pub current_hit_points: i32,
    pub maximum_hit_points: i32,
}

impl CharacterBody {
    pub async fn render(db: &Database, character: Character) -> Result<CharacterBody, Error> {
        let mut equipment = vec![];
        for entry in character.equipment {
            let item = item::db::fetch_item_by_id(db, entry.item_id).await?.ok_or(
                Error::ItemDoesNotExist {
                    item_id: entry.item_id,
                },
            )?;
            let body = ItemBody::render(item);
            equipment.push(ItemWithQuantityBody {
                quantity: entry.quantity,
                item: body,
            });
        }

        Ok(CharacterBody {
            id: character.id,
            owner: character.owner,
            name: character.name,
            created_at: character.created_at,
            modified_at: character.modified_at,
            stats: character.stats,
            equipment: equipment,
            position: character.position,
            current_hit_points: character.current_hit_points,
            maximum_hit_points: character.maximum_hit_points,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemWithQuantityBody {
    pub quantity: i32,
    pub item: ItemBody,
}

#[post("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn create_character_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<CreateCharacterBody>,
) -> Result<Json<CharacterBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    campaign::db::assert_campaign_exists(&db, campaign_id).await?;

    let now = Utc::now();
    let mut character = Character {
        id: CharacterId::new(),
        owner: CharacterOwner::Campaign(campaign_id),
        name: body.name,
        created_at: now,
        modified_at: now,
        stats: Default::default(),
        equipment: vec![],
        position: None,
        current_hit_points: 10,
        maximum_hit_points: 10,
        race: Race::Human,
        proficiencies: Proficiencies {
            armor: vec![],
            tool: vec![],
            saving_throws: vec![],
            skills: vec![],
        },
    };
    character.recalculate_stats(&db).await?;

    db::insert_character(&db, &character).await?;

    Ok(Json(CharacterBody::render(&db, character).await?))
}

#[get("/campaigns/{campaign_id}/characters")]
#[tracing::instrument(skip(db))]
async fn get_characters_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<CharacterBody>>, Error> {
    let campaign_id = params.into_inner();

    campaign::db::assert_campaign_exists(&db, campaign_id).await?;

    let characters = db::fetch_characters_by_campaign(&db, campaign_id).await?;

    let body = stream::iter(characters)
        .then(|character| CharacterBody::render(&db, character))
        .try_collect()
        .await?;

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/characters/{character_id}")]
#[tracing::instrument(skip(db))]
async fn get_character_in_campaign_by_id(
    db: Data<Database>,
    params: Path<(CampaignId, CharacterId)>,
) -> Result<Json<CharacterBody>, Error> {
    let (campaign_id, character_id) = params.into_inner();

    campaign::db::assert_campaign_exists(&db, campaign_id).await?;

    let character = db::fetch_character_by_campaign_and_id(&db, campaign_id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExistInCampaign {
            campaign_id,
            character_id,
        })?;

    Ok(Json(CharacterBody::render(&db, character).await?))
}

#[derive(Clone, Debug, Serialize)]
pub struct RollStatsBody {
    modifier: RollModifier,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum RollModifier {
    Advantage,
    Normal,
    Disadvantage,
}

#[get("/campaigns/{campaign_id}/characters/{character_id}/roll/{roll_type}")]
#[tracing::instrument(skip(db))]
async fn get_character_roll_stats(
    db: Data<Database>,
    params: Path<(CampaignId, CharacterId, RollType)>,
) -> Result<Json<RollStatsBody>, Error> {
    let (campaign_id, character_id, roll_type) = params.into_inner();
    let campaign = campaign::db::assert_campaign_exists(&db, campaign_id).await?;
    let character = db::fetch_character_by_campaign_and_id(&db, campaign.id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExistInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    let mut stats = RollStatsBody {
        modifier: RollModifier::Normal,
    };
    match roll_type {
        RollType::SkillCheck(skill) => {
            if character.proficiencies.skills.contains(&skill) {
                stats.modifier = RollModifier::Advantage;
            }
        }
        RollType::Save(ability) => {
            if character.proficiencies.saving_throws.contains(&ability) {
                stats.modifier = RollModifier::Advantage;
            }
        }
        _ => {}
    }

    let ability = match roll_type {
        RollType::SkillCheck(skill) => Some(skill.ability()),
        RollType::AbilityCheck(ability) => Some(ability),
        RollType::Save(ability) => Some(ability),
        _ => None,
    };

    if matches!(
        ability,
        Some(AbilityType::Strength) | Some(AbilityType::Dexterity)
    ) || matches!(roll_type, RollType::Hit)
    {
        let items: Vec<_> = stream::iter(&character.equipment)
            .then(|equipment| item::db::fetch_item_by_id(&db, equipment.item_id))
            .try_collect()
            .await?;

        for item in items {
            if let Some(item) = item {
                if let item::ItemType::Armor(armor) = item.item_type {
                    if !character.proficiencies.armor.contains(&armor.armor_type) {
                        // TODO: cancel out advantage
                        stats.modifier = RollModifier::Disadvantage;
                    }
                }
            }
        }
    }

    Err(Error::PathDoesNotExist)
}
