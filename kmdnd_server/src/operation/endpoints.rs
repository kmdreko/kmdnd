use std::vec;

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::campaign::CampaignId;
use crate::character::{CharacterId, Position};
use crate::database::Database;
use crate::encounter::{EncounterId, EncounterState};
use crate::error::Error;
use crate::item::{DamageType, ItemId};
use crate::operation::attack::{Attack, AttackMethod};
use crate::operation::spell::Cast;
use crate::operation::{Action, Interaction, InteractionId, Legality};
use crate::utils::SuccessBody;
use crate::violations::Violation;

use super::{Move, Operation, OperationId, OperationType, RollType, SpellTarget};

#[derive(Clone, Debug, Serialize)]
pub struct OperationBody {
    pub id: OperationId,
    pub campaign_id: CampaignId,
    pub encounter_id: Option<EncounterId>,
    pub encounter_state: Option<EncounterState>,
    pub character_id: CharacterId,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub operation_type: OperationType,
    pub interactions: Vec<Interaction>,
    pub legality: Legality,
}

impl OperationBody {
    fn render(operation: Operation) -> OperationBody {
        OperationBody {
            id: operation.id,
            campaign_id: operation.campaign_id,
            encounter_id: operation.encounter_id,
            encounter_state: operation.encounter_state,
            character_id: operation.character_id,
            created_at: operation.created_at,
            modified_at: operation.modified_at,
            operation_type: operation.operation_type,
            interactions: operation.interactions,
            legality: operation.legality,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MoveBody {
    pub character_id: CharacterId,
    pub position: Position,
    pub ignore_violations: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActionBody {
    pub character_id: CharacterId,
    pub action_type: ActionTypeBody,
    pub ignore_violations: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ActionTypeBody {
    Attack(AttackBody),
    CastSpell(CastBody),
    Dash,
    Disengage,
    Dodge,
    Help,
    Hide,
    Ready,
    Search,
    UseObject,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AttackBody {
    pub target_character_id: CharacterId,
    pub method: AttackMethodBody,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CastBody {
    pub name: String,
    pub target: SpellTarget,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum AttackMethodBody {
    Unarmed { damage_type: DamageType },
    Weapon { weapon_id: ItemId },
    ImprovisedWeapon { weapon_id: ItemId },
}

impl AttackMethodBody {
    async fn into_attack_method(self, db: &dyn Database) -> Result<AttackMethod, Error> {
        let attack_method = match self {
            AttackMethodBody::Unarmed { damage_type } => AttackMethod::Unarmed(damage_type),
            AttackMethodBody::Weapon { weapon_id } => {
                let item = db
                    .items()
                    .fetch_item_by_id(weapon_id)
                    .await?
                    .ok_or(Error::ItemDoesNotExist { item_id: weapon_id })?;

                let weapon = item
                    .item_type
                    .into_weapon()
                    .ok_or(Error::ItemIsNotAWeapon { item_id: item.id })?;

                AttackMethod::Weapon(weapon)
            }
            AttackMethodBody::ImprovisedWeapon { weapon_id } => {
                let item = db
                    .items()
                    .fetch_item_by_id(weapon_id)
                    .await?
                    .ok_or(Error::ItemDoesNotExist { item_id: weapon_id })?;

                let weapon = item
                    .item_type
                    .into_weapon()
                    .ok_or(Error::ItemIsNotAWeapon { item_id: item.id })?;

                AttackMethod::ImprovisedWeapon(weapon)
            }
        };

        Ok(attack_method)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RollBody {
    pub character_id: CharacterId,
    pub roll: RollType,
}

#[derive(Clone, Debug, Serialize)]
pub struct RollResultBody {
    result: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct BeginEncounterResultBody {
    turn_order: Vec<CharacterId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubmitInteractionBody {
    interaction_id: InteractionId,
    character_id: CharacterId,
    result: i32,
}

#[get("/campaigns/{campaign_id}/encounters/CURRENT/operations")]
#[tracing::instrument(skip(db))]
async fn get_operations_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<OperationBody>>, Error> {
    let campaign_id = params.into_inner();

    db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign_id)
        .await?;

    let operations = db
        .operations()
        .fetch_operations_by_encounter(encounter.id)
        .await?;

    let body = operations.into_iter().map(OperationBody::render).collect();

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}")]
#[tracing::instrument(skip(db))]
async fn get_operation_by_id_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, OperationId)>,
) -> Result<Json<OperationBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let operation = db
        .operations()
        .fetch_operation_by_id(operation_id)
        .await?
        .ok_or(Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        })?;

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}/approve")]
#[tracing::instrument(skip(db))]
async fn approve_illegal_operation(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, OperationId)>,
) -> Result<Json<SuccessBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let operation = db
        .operations()
        .fetch_operation_by_id(operation_id)
        .await?
        .ok_or(Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        })?;

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

    Ok(Json(SuccessBody {}))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}/reject")]
#[tracing::instrument(skip(db))]
async fn reject_illegal_operation(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, OperationId)>,
) -> Result<Json<SuccessBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let operation = db
        .operations()
        .fetch_operation_by_id(operation_id)
        .await?
        .ok_or(Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        })?;

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

    Ok(Json(SuccessBody {}))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}/interactions")]
#[tracing::instrument(skip(db))]
async fn submit_interaction_result_to_operation(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, OperationId)>,
    body: Json<SubmitInteractionBody>,
) -> Result<Json<OperationBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let mut operation = db
        .operations()
        .fetch_operation_by_id(operation_id)
        .await?
        .ok_or(Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        })?;

    let (index, interaction) = operation
        .interactions
        .iter()
        .enumerate()
        .find(|(_, inter)| inter.id == body.interaction_id)
        .ok_or(Error::InteractionDoesNotExist {
            operation_id: operation.id,
            interaction_id: body.interaction_id,
        })?;

    if interaction.character_id != body.character_id {
        return Err(Error::WrongCharacterForInteraction {
            operation_id: operation.id,
            interaction_id: interaction.id,
            expected_character_id: interaction.character_id,
            request_character_id: body.character_id,
        });
    }

    let new_interactions = match &operation.operation_type {
        OperationType::Action(action) => match action {
            Action::Attack(attack) => {
                attack
                    .handle_interaction_result(&***db, campaign.id, &interaction, body.result)
                    .await?
            }
            Action::CastSpell(cast) => {
                cast.handle_interaction_result(
                    &***db,
                    campaign.id,
                    &encounter,
                    &operation,
                    &interaction,
                    body.result,
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
        .update_operation_interaction_result(operation, index, body.result)
        .await?;
    if !new_interactions.is_empty() {
        operation = db
            .operations()
            .update_operation_push_interactions(operation, new_interactions)
            .await?;
    }

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/roll")]
#[tracing::instrument(skip(db))]
async fn roll_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<RollBody>,
) -> Result<Json<RollResultBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let character = db
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

    let operations = db
        .operations()
        .fetch_operations_by_encounter(encounter.id)
        .await?;
    let character_already_rolled =
        operations
            .iter()
            .any(|operation| match operation.operation_type {
                OperationType::Roll { roll, .. } => {
                    operation.character_id == body.character_id && roll == RollType::Initiative
                }
                _ => false,
            });
    if character_already_rolled {
        return Err(Error::CharacterAlreadyRolledInitiative {
            campaign_id: campaign.id,
            encounter_id: encounter.id,
            character_id: body.character_id,
        });
    }

    let result = rand::thread_rng().gen_range(1..=20) + character.stats.initiative;

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign.id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state),
        character_id: body.character_id,
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

    Ok(Json(RollResultBody { result }))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/begin")]
#[tracing::instrument(skip(db))]
async fn begin_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
) -> Result<Json<BeginEncounterResultBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

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
        campaign_id,
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

    let body = BeginEncounterResultBody {
        turn_order: encounter.character_ids,
    };

    Ok(Json(body))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/move")]
#[tracing::instrument(skip(db))]
async fn move_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<MoveBody>,
) -> Result<Json<OperationBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let body = body.into_inner();

    let current_character = db
        .characters()
        .fetch_character_by_campaign_and_id(campaign_id, body.character_id)
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

    let current_position =
        current_character
            .position
            .as_ref()
            .ok_or(Error::CharacterDoesNotHavePosition {
                character_id: body.character_id,
            })?;

    let desired_position = body.position;
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

    if !body.ignore_violations && !violations.is_empty() {
        return Err(Error::OperationViolatesRules { violations });
    }

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign.id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state),
        character_id: body.character_id,
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
        .update_character_position(current_character, Some(body.position))
        .await?;

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/action")]
#[tracing::instrument(skip(db))]
async fn take_action_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<ActionBody>,
) -> Result<Json<OperationBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = db.campaigns().assert_campaign_exists(campaign_id).await?;
    let encounter = db
        .encounters()
        .assert_current_encounter_exists(campaign.id)
        .await?;

    let body = body.into_inner();

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
            let attack_method = attack.method.into_attack_method(&***db).await?;

            let (attack, interactions, violations) = Attack::submit(
                &***db,
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
                &***db,
                campaign_id,
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
        encounter_state: Some(encounter.state),
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

    Ok(Json(OperationBody::render(operation)))
}
