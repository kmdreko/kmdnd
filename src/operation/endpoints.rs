use std::vec;

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use mongodb::Database;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::{self, CharacterId, Position};
use crate::encounter::{self, EncounterId, EncounterState};
use crate::error::Error;
use crate::item::{self, DamageType, ItemId};
use crate::operation::{Action, Attack, AttackMethod, Interaction, InteractionId};

use super::{db, Move, Operation, OperationId, OperationType, RollType};

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
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MoveBody {
    pub character_id: CharacterId,
    pub position: Position,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActionBody {
    pub character_id: CharacterId,
    pub action_type: ActionTypeBody,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ActionTypeBody {
    Melee,
    Attack(AttackBody),
    CastSpell,
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
#[serde(tag = "type", rename_all = "SCREAMING-KEBAB-CASE")]
pub enum AttackMethodBody {
    Unarmed { damage_type: DamageType },
    Weapon { weapon_id: ItemId },
    ImprovisedWeapon { weapon_id: ItemId },
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
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<OperationBody>>, Error> {
    let campaign_id = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let operations = db::fetch_operations_by_encounter(&db, encounter.id).await?;

    let body = operations
        .into_iter()
        .map(|operation| OperationBody::render(operation))
        .collect();

    Ok(Json(body))
}

#[get("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}")]
#[tracing::instrument(skip(db))]
async fn get_operation_by_id_in_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<(CampaignId, OperationId)>,
) -> Result<Json<OperationBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let operation = db::fetch_operation_by_id(&db, operation_id).await?.ok_or(
        Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        },
    )?;

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}/interactions")]
#[tracing::instrument(skip(db))]
async fn submit_interaction_result_to_operation(
    db: Data<Database>,
    params: Path<(CampaignId, OperationId)>,
    body: Json<SubmitInteractionBody>,
) -> Result<Json<OperationBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let mut operation = db::fetch_operation_by_id(&db, operation_id).await?.ok_or(
        Error::OperationDoesNotExist {
            encounter_id: encounter.id,
            operation_id,
        },
    )?;

    let (index, interaction) = operation
        .interactions
        .iter_mut()
        .enumerate()
        .find(|(_, inter)| inter.id == body.interaction_id)
        .ok_or(Error::InteractionDoesNotExist {
            operation_id,
            interaction_id: body.interaction_id,
        })?;

    if interaction.character_id != body.character_id {
        return Err(Error::WrongCharacterForInteraction {
            operation_id: operation_id,
            interaction_id: interaction.id,
            expected_character_id: interaction.character_id,
            request_character_id: body.character_id,
        });
    }

    let new_interactions = match &operation.operation_type {
        OperationType::Action(action) => match action {
            Action::Attack(attack) => {
                match interaction.roll_type {
                    RollType::Hit => {
                        let target_character_id = attack.targets[0]; // TODO:
                        let target_character = character::db::fetch_character_by_campaign_and_id(
                            &db,
                            campaign_id,
                            target_character_id,
                        )
                        .await?
                        .ok_or(Error::CharacterDoesNotExistInCampaign {
                            campaign_id,
                            character_id: target_character_id,
                        })?;

                        if target_character.stats.armor_class <= body.result {
                            vec![Interaction {
                                id: InteractionId::new(),
                                character_id: interaction.character_id,
                                roll_type: RollType::Damage,
                                result: None,
                            }]
                        } else {
                            vec![]
                        }
                    }
                    RollType::Damage => {
                        // TODO: do actual damage
                        vec![]
                    }
                    _ => {
                        vec![]
                    }
                }
            }
            _ => {
                vec![]
            }
        },
        _ => {
            vec![]
        }
    };

    interaction.result = Some(body.result);

    let updated_at =
        db::update_operation_interaction_result(&db, &operation, index, body.result).await?;
    operation.modified_at = updated_at;
    if !new_interactions.is_empty() {
        db::update_operation_push_interactions(&db, &operation, &new_interactions).await?;
    }

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/roll")]
#[tracing::instrument(skip(db))]
async fn roll_in_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<RollBody>,
) -> Result<Json<RollResultBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let character =
        character::db::fetch_character_by_campaign_and_id(&db, campaign_id, body.character_id)
            .await?
            .ok_or(Error::CharacterNotInCampaign {
                campaign_id,
                character_id: body.character_id,
            })?;

    if !encounter.character_ids.contains(&body.character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id,
            encounter_id: encounter.id,
            character_id: body.character_id,
        });
    }

    let operations = db::fetch_operations_by_encounter(&db, encounter.id).await?;
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
            campaign_id,
            encounter_id: encounter.id,
            character_id: body.character_id,
        });
    }

    let result = rand::thread_rng().gen_range(1..=20) + character.stats.initiative;

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign_id,
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
    };

    db::insert_operation(&db, &operation).await?;

    Ok(Json(RollResultBody { result }))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/begin")]
#[tracing::instrument(skip(db))]
async fn begin_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<BeginEncounterResultBody>, Error> {
    let campaign_id = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let operations = db::fetch_operations_by_encounter(&db, encounter.id).await?;
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

    if uninitiated_character_ids.len() > 0 {
        return Err(Error::CharactersHaveNotRolledInitiative {
            campaign_id,
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

    encounter::db::update_encounter_state_and_characters(
        &db,
        &encounter,
        EncounterState::Turn {
            round: 0,
            character_id: *first_character,
        },
        &turn_order,
    )
    .await?;

    let body = BeginEncounterResultBody { turn_order };

    Ok(Json(body))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/move")]
#[tracing::instrument(skip(db))]
async fn move_in_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<MoveBody>,
) -> Result<Json<OperationBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let current_character =
        character::db::fetch_character_by_campaign_and_id(&db, campaign_id, body.character_id)
            .await?
            .ok_or(Error::CharacterNotInCampaign {
                campaign_id,
                character_id: body.character_id,
            })?;

    if !encounter.character_ids.contains(&body.character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id,
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

    if let EncounterState::Turn {
        round,
        character_id,
    } = encounter.state
    {
        if current_character.id != character_id {
            return Err(Error::NotThisPlayersTurn {
                campaign_id,
                encounter_id: encounter.id,
                current_character_id: character_id,
                request_character_id: current_character.id,
            });
        }

        let operations =
            db::fetch_operations_by_turn(&db, encounter.id, round, character_id).await?;

        let already_moved_feet: f32 = operations
            .iter()
            .filter(|op| op.character_id == current_character.id)
            .filter_map(|op| op.operation_type.as_move())
            .map(|mov| mov.feet)
            .sum();

        let maximum_movement = current_character.stats.speed as f32;
        if maximum_movement < already_moved_feet + feet {
            return Err(Error::CharacterMovementExceeded {
                character_id,
                maximum_movement,
                current_movement: already_moved_feet,
                request_movement: feet,
            });
        }
    }

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign_id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state),
        character_id: body.character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Move(Move {
            to_position: desired_position,
            feet: feet,
        }),
        interactions: vec![],
    };

    db::insert_operation(&db, &operation).await?;
    character::db::update_character_position(&db, &current_character, Some(body.position)).await?;

    Ok(Json(OperationBody::render(operation)))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/action")]
#[tracing::instrument(skip(db))]
async fn take_action_in_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
    body: Json<ActionBody>,
) -> Result<Json<OperationBody>, Error> {
    let campaign_id = params.into_inner();
    let body = body.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist { campaign_id })?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist { campaign_id })?;

    let source_character =
        character::db::fetch_character_by_campaign_and_id(&db, campaign_id, body.character_id)
            .await?
            .ok_or(Error::CharacterNotInCampaign {
                campaign_id,
                character_id: body.character_id,
            })?;

    if !encounter.character_ids.contains(&body.character_id) {
        return Err(Error::CharacterNotInEncounter {
            campaign_id,
            encounter_id: encounter.id,
            character_id: body.character_id,
        });
    }

    if let EncounterState::Turn { character_id, .. } = encounter.state {
        if character_id != body.character_id {
            return Err(Error::NotThisPlayersTurn {
                campaign_id,
                encounter_id: encounter.id,
                request_character_id: body.character_id,
                current_character_id: character_id,
            });
        }
    }

    let (action, interactions) =
        match body.action_type {
            ActionTypeBody::Attack(attack) => {
                let target_character = character::db::fetch_character_by_campaign_and_id(
                    &db,
                    campaign_id,
                    attack.target_character_id,
                )
                .await?
                .ok_or(Error::CharacterNotInCampaign {
                    campaign_id,
                    character_id: attack.target_character_id,
                })?;

                if !encounter
                    .character_ids
                    .contains(&attack.target_character_id)
                {
                    return Err(Error::CharacterNotInEncounter {
                        campaign_id,
                        encounter_id: encounter.id,
                        character_id: attack.target_character_id,
                    });
                }

                let attack_method = match attack.method {
                    AttackMethodBody::Unarmed { damage_type } => AttackMethod::Unarmed(damage_type),
                    AttackMethodBody::Weapon { weapon_id } => {
                        let item = item::db::fetch_item_by_id(&db, weapon_id)
                            .await?
                            .ok_or(Error::ItemDoesNotExist { item_id: weapon_id })?;

                        let weapon = item
                            .item_type
                            .into_weapon()
                            .ok_or(Error::ItemIsNotAWeapon { item_id: item.id })?;

                        AttackMethod::Weapon(weapon)
                    }
                    AttackMethodBody::ImprovisedWeapon { weapon_id } => {
                        let item = item::db::fetch_item_by_id(&db, weapon_id)
                            .await?
                            .ok_or(Error::ItemDoesNotExist { item_id: weapon_id })?;

                        let weapon = item
                            .item_type
                            .into_weapon()
                            .ok_or(Error::ItemIsNotAWeapon { item_id: item.id })?;

                        AttackMethod::ImprovisedWeapon(weapon)
                    }
                };

                let source_position = source_character.position.as_ref().ok_or(
                    Error::CharacterDoesNotHavePosition {
                        character_id: source_character.id,
                    },
                )?;
                let target_position = target_character.position.as_ref().ok_or(
                    Error::CharacterDoesNotHavePosition {
                        character_id: target_character.id,
                    },
                )?;

                let attack_range = attack_method.normal_range();
                let current_range = Position::distance(source_position, target_position);
                if attack_range < current_range {
                    return Err(Error::AttackNotInRange {
                        request_character_id: source_character.id,
                        target_character_id: target_character.id,
                        attack_range,
                        current_range,
                    });
                }

                let interactions = vec![Interaction {
                    id: InteractionId::new(),
                    character_id: source_character.id,
                    roll_type: RollType::Hit,
                    result: None,
                }];

                let action = Action::Attack(Attack {
                    method: attack_method,
                    targets: vec![target_character.id],
                });

                (action, interactions)
            }
            _ => unimplemented!("the action is not yet implemented"),
        };

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign_id,
        encounter_id: Some(encounter.id),
        encounter_state: Some(encounter.state),
        character_id: body.character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Action(action),
        interactions,
    };

    db::insert_operation(&db, &operation).await?;

    Ok(Json(OperationBody::render(operation)))
}
