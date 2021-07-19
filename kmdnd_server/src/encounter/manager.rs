use chrono::Utc;

use crate::campaign::Campaign;
use crate::character::CharacterId;
use crate::database::Database;
use crate::error::Error;
use crate::operation::RollType;

use super::{Encounter, EncounterId, EncounterState};

#[tracing::instrument(skip(db))]
pub async fn create_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    character_ids: Vec<CharacterId>,
) -> Result<Encounter, Error> {
    let current_encounter = db
        .encounters()
        .fetch_current_encounter_by_campaign(campaign.id)
        .await?;
    if let Some(current_encounter) = current_encounter {
        return Err(Error::CurrentEncounterAlreadyExists {
            campaign_id: campaign.id,
            encounter_id: current_encounter.id,
        });
    }

    let characters = db
        .characters()
        .fetch_characters_by_campaign(campaign.id)
        .await?;
    for character_id in &character_ids {
        if !characters.iter().any(|c| c.id == *character_id) {
            return Err(Error::CharacterNotInCampaign {
                campaign_id: campaign.id,
                character_id: *character_id,
            });
        }
    }

    let now = Utc::now();
    let encounter = Encounter {
        id: EncounterId::new(),
        campaign_id: campaign.id,
        created_at: now,
        modified_at: now,
        character_ids,
        state: EncounterState::Initiative,
    };

    db.encounters().insert_encounter(&encounter).await?;

    Ok(encounter)
}

#[tracing::instrument(skip(db))]
pub async fn get_encounters_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
) -> Result<Vec<Encounter>, Error> {
    let encounters = db
        .encounters()
        .fetch_encounters_by_campaign(campaign.id)
        .await?;

    Ok(encounters)
}

#[tracing::instrument(skip(db))]
pub async fn get_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
) -> Result<Encounter, Error> {
    let encounter = db
        .encounters()
        .fetch_current_encounter_by_campaign(campaign.id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist {
            campaign_id: campaign.id,
        })?;

    Ok(encounter)
}

#[tracing::instrument(skip(db))]
pub async fn finish_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
) -> Result<(), Error> {
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    db.encounters()
        .update_encounter_state(encounter, EncounterState::Finished)
        .await?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn begin_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
) -> Result<Vec<CharacterId>, Error> {
    let encounter = db
        .encounters()
        .fetch_current_encounter_by_campaign(campaign.id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist {
            campaign_id: campaign.id,
        })?;

    let operations = db
        .operations()
        .fetch_operations_by_encounter(encounter.id)
        .await?;
    let mut initiative_rolls: Vec<(CharacterId, i32)> = operations
        .iter()
        .filter_map(|operation| {
            operation
                .operation_type
                .as_roll()
                .map(|(roll, result)| (operation.character_id, roll, result))
        })
        .filter(|(_, roll, _)| *roll == RollType::Initiative)
        .map(|(character_id, _, result)| (character_id, result))
        .collect();

    let uninitiated_character_ids: Vec<_> = encounter
        .character_ids
        .iter()
        .copied()
        .filter(|character_id| {
            !initiative_rolls
                .iter()
                .any(|(c_id, _)| c_id == character_id)
        })
        .collect();

    if !uninitiated_character_ids.is_empty() {
        return Err(Error::CharactersHaveNotRolledInitiative {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_ids: uninitiated_character_ids,
        });
    }

    initiative_rolls.sort_by_key(|(_, result)| *result);
    initiative_rolls.reverse();
    let turn_order: Vec<_> = initiative_rolls
        .into_iter()
        .map(|(character_id, _)| character_id)
        .collect();

    let first_character = turn_order.first().ok_or(Error::NoCharactersInEncounter {
        campaign_id: campaign.id,
        encounter_id: encounter.id,
    })?;

    let encounter = db
        .encounters()
        .update_encounter_state_and_characters(
            encounter,
            EncounterState::Turn {
                round: 0,
                character_id: *first_character,
            },
            turn_order,
        )
        .await?;

    Ok(encounter.character_ids)
}
