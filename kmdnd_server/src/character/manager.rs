use chrono::Utc;
use futures::{future, stream, StreamExt, TryStreamExt};

use crate::campaign::Campaign;
use crate::character::race::Race;
use crate::character::{Proficiencies, RollModifier};
use crate::database::Database;
use crate::error::Error;
use crate::item::{self};
use crate::operation::{AbilityType, RollType};

use super::{Character, CharacterId, CharacterOwner};

#[tracing::instrument(skip(db))]
pub async fn create_character_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    name: String,
) -> Result<Character, Error> {
    let now = Utc::now();
    let mut character = Character {
        id: CharacterId::new(),
        owner: CharacterOwner::Campaign(campaign.id),
        name,
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
        racial_traits: vec![],
    };
    character.recalculate_stats(db).await?;

    db.characters().insert_character(&character).await?;

    Ok(character)
}

#[tracing::instrument(skip(db))]
pub async fn get_characters_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
) -> Result<Vec<Character>, Error> {
    let characters = db
        .characters()
        .fetch_characters_by_campaign(campaign.id)
        .await?;

    Ok(characters)
}

#[tracing::instrument(skip(db))]
pub async fn get_character_in_campaign_by_id(
    db: &dyn Database,
    campaign: &Campaign,
    character_id: CharacterId,
) -> Result<Character, Error> {
    let character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign.id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExistInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    Ok(character)
}

#[tracing::instrument(skip(db))]
pub async fn get_character_roll_stats(
    db: &dyn Database,
    campaign: &Campaign,
    character_id: CharacterId,
    roll_type: RollType,
) -> Result<RollModifier, Error> {
    let character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign.id, character_id)
        .await?
        .ok_or(Error::CharacterDoesNotExistInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    let mut modifier = RollModifier::Normal;
    match roll_type {
        RollType::SkillCheck(skill) => {
            if character.proficiencies.skills.contains(&skill) {
                modifier = RollModifier::Advantage;
            }
        }
        RollType::Save(ability) => {
            if character.proficiencies.saving_throws.contains(&ability) {
                modifier = RollModifier::Advantage;
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
            .then(|equipment| db.items().fetch_item_by_id(equipment.item_id))
            .try_filter_map(|item| future::ready(Ok(item)))
            .try_collect()
            .await?;

        for item in items {
            if let item::ItemType::Armor(armor) = item.item_type {
                if !character.proficiencies.armor.contains(&armor.armor_type) {
                    // TODO: cancel out advantage
                    modifier = RollModifier::Disadvantage;
                }
            }
        }
    }

    Ok(modifier)
}
