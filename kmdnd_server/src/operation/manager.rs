use chrono::Utc;
use rand::Rng;

use crate::campaign::Campaign;
use crate::character::{CharacterId, Position};
use crate::database::Database;
use crate::encounter::{Encounter, EncounterState};
use crate::error::Error;
use crate::operation::attack::Attack;
use crate::operation::spell::Cast;
use crate::operation::{Action, ActionTypeBody, InteractionId, Legality};
use crate::violations::Violation;

use super::{ActionBody, Move, Operation, OperationId, OperationType, RollType};

#[tracing::instrument(skip(db))]
pub async fn get_operations_in_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
) -> Result<Vec<Operation>, Error> {
    let operations = db
        .operations()
        .fetch_operations_by_encounter(encounter.id)
        .await?;

    Ok(operations)
}

#[tracing::instrument(skip(db))]
pub async fn get_operation_by_id_in_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    operation_id: OperationId,
) -> Result<Option<Operation>, Error> {
    let operation = db.operations().fetch_operation_by_id(operation_id).await?;

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn approve_illegal_operation(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    operation: Operation,
) -> Result<(), Error> {
    match operation.legality.clone() {
        Legality::IllegalPending { violations } => {
            db.operations()
                .update_operation_legality(operation, Legality::IllegalApproved { violations })
                .await?;
        }
        legality => {
            return Err(Error::OperationIsNotPending {
                operation_id: operation.id,
                legality,
            });
        }
    }

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn reject_illegal_operation(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    operation: Operation,
) -> Result<(), Error> {
    match operation.legality.clone() {
        Legality::IllegalPending { .. } => {
            db.operations().delete_operation(operation.id).await?;
        }
        legality => {
            return Err(Error::OperationIsNotPending {
                operation_id: operation.id,
                legality,
            });
        }
    }

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn submit_interaction_result_to_operation(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    mut operation: Operation,
    interaction_id: InteractionId,
    character_id: CharacterId,
    result: i32,
) -> Result<Operation, Error> {
    let (index, interaction) = operation
        .interactions
        .iter()
        .enumerate()
        .find(|(_, inter)| inter.id == interaction_id)
        .ok_or(Error::InteractionNotFound {
            operation_id: operation.id,
            interaction_id,
        })?;

    if interaction.character_id != character_id {
        return Err(Error::WrongCharacterForInteraction {
            operation_id: operation.id,
            interaction_id: interaction.id,
            expected_character_id: interaction.character_id,
            request_character_id: character_id,
        });
    }

    let new_interactions = match &operation.operation_type {
        OperationType::Action(action) => match action {
            Action::Attack(attack) => {
                attack
                    .handle_interaction_result(db, &campaign, &interaction, result)
                    .await?
            }
            Action::CastSpell(cast) => {
                cast.handle_interaction_result(
                    db,
                    &campaign,
                    &encounter,
                    &operation,
                    &interaction,
                    result,
                )
                .await?
            }
            _ => {
                vec![]
            }
        },
        _ => {
            vec![]
        }
    };

    operation = db
        .operations()
        .update_operation_interaction_result(operation, index, result)
        .await?;
    if !new_interactions.is_empty() {
        operation = db
            .operations()
            .update_operation_push_interactions(operation, new_interactions)
            .await?;
    }

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn roll_in_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    character_id: CharacterId,
    roll: RollType,
) -> Result<i32, Error> {
    let character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign.id, character_id)
        .await?
        .ok_or(Error::CharacterNotInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    if !encounter.character_ids.contains(&character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_id,
        });
    }

    let operations = db
        .operations()
        .fetch_operations_by_encounter(encounter.id)
        .await?;
    let character_already_rolled =
        operations
            .iter()
            .any(|operation| match operation.operation_type {
                OperationType::Roll { roll, .. } => {
                    operation.character_id == character_id && roll == RollType::Initiative
                }
                _ => false,
            });
    if character_already_rolled {
        return Err(Error::CharacterAlreadyRolledInitiative {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_id,
        });
    }

    let result = rand::thread_rng().gen_range(1..=20) + character.stats.initiative;

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign.id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state.clone()),
        character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Roll {
            roll: RollType::Initiative,
            result,
        },
        interactions: vec![],
        legality: Legality::Legal,
    };

    db.operations().insert_operation(&operation).await?;

    Ok(result)
}

#[tracing::instrument(skip(db))]
pub async fn move_in_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    character_id: CharacterId,
    position: Position,
    ignore_violations: bool,
) -> Result<Operation, Error> {
    let current_character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign.id, character_id)
        .await?
        .ok_or(Error::CharacterNotInCampaign {
            campaign_id: campaign.id,
            character_id,
        })?;

    if !encounter.character_ids.contains(&character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_id,
        });
    }

    let current_position = current_character
        .position
        .as_ref()
        .ok_or(Error::CharacterDoesNotHavePosition { character_id })?;

    let desired_position = position;
    let feet = Position::distance(&current_position, &desired_position);
    let mut violations = vec![];

    if let EncounterState::Turn {
        round,
        character_id,
    } = encounter.state
    {
        if current_character.id != character_id {
            return Err(Error::NotThisPlayersTurn {
                campaign_id: campaign.id,
                encounter_id: encounter.id,
                current_character_id: character_id,
                request_character_id: current_character.id,
            });
        }

        let operations = db
            .operations()
            .fetch_operations_by_turn(encounter.id, round, character_id)
            .await?;

        let already_moved_feet: f32 = operations
            .iter()
            .filter(|op| op.character_id == current_character.id)
            .filter_map(|op| op.operation_type.as_move())
            .map(|mov| mov.feet)
            .sum();

        let maximum_movement = current_character.stats.speed as f32;
        if maximum_movement < already_moved_feet + feet {
            violations.push(Violation::CharacterMovementExceeded {
                character_id,
                maximum_movement,
                current_movement: already_moved_feet,
                request_movement: feet,
            });
        }
    }

    if !ignore_violations && !violations.is_empty() {
        return Err(Error::OperationViolatesRules { violations });
    }

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign.id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state.clone()),
        character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Move(Move {
            to_position: desired_position,
            feet,
        }),
        interactions: vec![],
        legality: if violations.is_empty() {
            Legality::Legal
        } else {
            Legality::IllegalPending { violations }
        },
    };

    db.operations().insert_operation(&operation).await?;
    db.characters()
        .update_character_position(current_character, Some(position))
        .await?;

    Ok(operation)
}

#[tracing::instrument(skip(db))]
pub async fn take_action_in_current_encounter_in_campaign(
    db: &dyn Database,
    campaign: &Campaign,
    encounter: &Encounter,
    body: ActionBody,
) -> Result<Operation, Error> {
    let source_character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign.id, body.character_id)
        .await?
        .ok_or(Error::CharacterNotInCampaign {
            campaign_id: campaign.id,
            character_id: body.character_id,
        })?;

    if !encounter.character_ids.contains(&body.character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_id: body.character_id,
        });
    }

    if let EncounterState::Turn { character_id, .. } = encounter.state {
        if character_id != body.character_id {
            return Err(Error::NotThisPlayersTurn {
                campaign_id: campaign.id,
                encounter_id: encounter.id,
                request_character_id: body.character_id,
                current_character_id: character_id,
            });
        }
    }

    let (action, interactions, violations) = match body.action_type {
        ActionTypeBody::Attack(attack) => {
            let attack_method = attack.method.into_attack_method(db).await?;

            let (attack, interactions, violations) = Attack::submit(
                db,
                campaign.id,
                &encounter,
                source_character,
                attack.target_character_id,
                attack_method,
            )
            .await?;

            (Action::Attack(attack), interactions, violations)
        }
        ActionTypeBody::CastSpell(cast) => {
            let (cast, interactions, violations) = Cast::submit(
                db,
                campaign.id,
                &encounter,
                source_character,
                cast.name,
                cast.target,
            )
            .await?;

            (Action::CastSpell(cast), interactions, violations)
        }
        _ => unimplemented!("the action is not yet implemented"),
    };

    if !body.ignore_violations && !violations.is_empty() {
        return Err(Error::OperationViolatesRules { violations });
    }

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign.id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state.clone()),
        character_id: body.character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Action(action),
        interactions,
        legality: if violations.is_empty() {
            Legality::Legal
        } else {
            Legality::IllegalPending { violations }
        },
    };

    db.operations().insert_operation(&operation).await?;

    Ok(operation)
}
