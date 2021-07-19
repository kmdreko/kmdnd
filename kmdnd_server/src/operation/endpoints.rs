use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::{CharacterId, Position};
use crate::database::Database;
use crate::encounter::{self, EncounterId, EncounterState};
use crate::error::Error;
use crate::item::{self, DamageType, ItemId};
use crate::operation::attack::AttackMethod;
use crate::operation::{Interaction, InteractionId, Legality};
use crate::utils::SuccessBody;

use super::{manager, Operation, OperationId, OperationType, RollType, SpellTarget};

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
    pub async fn into_attack_method(self, db: &dyn Database) -> Result<AttackMethod, Error> {
        let attack_method = match self {
            AttackMethodBody::Unarmed { damage_type } => AttackMethod::Unarmed(damage_type),
            AttackMethodBody::Weapon { weapon_id } => {
                let item = item::manager::get_item_by_id(db, weapon_id)
                    .await?
                    .ok_or(Error::ItemDoesNotExist { item_id: weapon_id })?;

                let weapon = item
                    .item_type
                    .into_weapon()
                    .ok_or(Error::ItemIsNotAWeapon { item_id: item.id })?;

                AttackMethod::Weapon(weapon)
            }
            AttackMethodBody::ImprovisedWeapon { weapon_id } => {
                let item = item::manager::get_item_by_id(db, weapon_id)
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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;

    let operations =
        manager::get_operations_in_current_encounter_in_campaign(&***db, &campaign, &encounter)
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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let operation = manager::get_operation_by_id_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        operation_id,
    )
    .await?
    .ok_or(Error::OperationNotFound {
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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let operation = manager::get_operation_by_id_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        operation_id,
    )
    .await?
    .ok_or(Error::OperationNotFound {
        encounter_id: encounter.id,
        operation_id,
    })?;

    manager::approve_illegal_operation(&***db, &campaign, &encounter, operation).await?;

    Ok(Json(SuccessBody {}))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/operations/{operation_id}/reject")]
#[tracing::instrument(skip(db))]
async fn reject_illegal_operation(
    db: Data<Box<dyn Database>>,
    params: Path<(CampaignId, OperationId)>,
) -> Result<Json<SuccessBody>, Error> {
    let (campaign_id, operation_id) = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let operation = manager::get_operation_by_id_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        operation_id,
    )
    .await?
    .ok_or(Error::OperationNotFound {
        encounter_id: encounter.id,
        operation_id,
    })?;

    manager::reject_illegal_operation(&***db, &campaign, &encounter, operation).await?;

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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let operation = manager::get_operation_by_id_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        operation_id,
    )
    .await?
    .ok_or(Error::OperationNotFound {
        encounter_id: encounter.id,
        operation_id,
    })?;

    let operation = manager::submit_interaction_result_to_operation(
        &***db,
        &campaign,
        &encounter,
        operation,
        body.interaction_id,
        body.character_id,
        body.result,
    )
    .await?;

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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let body = body.into_inner();

    let result = manager::roll_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        body.character_id,
        body.roll,
    )
    .await?;

    Ok(Json(RollResultBody { result }))
}

#[post("/campaigns/{campaign_id}/encounters/CURRENT/move")]
#[tracing::instrument(skip(db))]
async fn move_in_current_encounter_in_campaign(
    db: Data<Box<dyn Database>>,
    params: Path<CampaignId>,
    body: Json<MoveBody>,
) -> Result<Json<OperationBody>, Error> {
    let campaign_id = params.into_inner();
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let body = body.into_inner();

    let operation = manager::move_in_current_encounter_in_campaign(
        &***db,
        &campaign,
        &encounter,
        body.character_id,
        body.position,
        body.ignore_violations,
    )
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
    let campaign = campaign::manager::get_campaign_by_id(&***db, campaign_id)
        .await?
        .ok_or(Error::CampaignNotFound { campaign_id })?;
    let encounter = encounter::manager::get_current_encounter_in_campaign(&***db, &campaign)
        .await?
        .ok_or(Error::CurrentEncounterNotFound {
            campaign_id: campaign.id,
        })?;
    let body = body.into_inner();

    let operation =
        manager::take_action_in_current_encounter_in_campaign(&***db, &campaign, &encounter, body)
            .await?;

    Ok(Json(OperationBody::render(operation)))
}
