use actix_web::web::{Data, Json, Path};
use actix_web::{get, post};
use chrono::{DateTime, Utc};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::campaign::{self, CampaignId};
use crate::character::CharacterId;
use crate::encounter::{self, EncounterId};
use crate::error::Error;

use super::{db, Operation, OperationId, OperationType};

#[derive(Clone, Debug, Serialize)]
pub struct OperationBody {
    pub id: OperationId,
    pub campaign_id: CampaignId,
    pub encounter_id: Option<EncounterId>,
    pub character_id: CharacterId,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub operation_type: OperationType,
}

impl OperationBody {
    fn render(operation: Operation) -> OperationBody {
        OperationBody {
            id: operation.id,
            campaign_id: operation.campaign_id,
            encounter_id: operation.encounter_id,
            character_id: operation.character_id,
            created_at: operation.created_at,
            modified_at: operation.modified_at,
            operation_type: operation.operation_type,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MoveBody {
    pub character_id: CharacterId,
    pub feet: f32,
}

#[get("/campaigns/{campaign_id}/encounters/current/operations")]
#[tracing::instrument(skip(db))]
async fn get_operations_in_current_encounter_in_campaign(
    db: Data<Database>,
    params: Path<CampaignId>,
) -> Result<Json<Vec<OperationBody>>, Error> {
    let campaign_id = params.into_inner();

    campaign::db::fetch_campaign_by_id(&db, campaign_id)
        .await?
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    let operations = db::fetch_operations_by_encounter(&db, encounter.id).await?;

    let body = operations
        .into_iter()
        .map(|operation| OperationBody::render(operation))
        .collect();

    Ok(Json(body))
}

#[post("/campaigns/{campaign_id}/encounters/current/move")]
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
        .ok_or(Error::CampaignDoesNotExist(campaign_id))?;

    let encounter = encounter::db::fetch_current_encounter_by_campaign(&db, campaign_id)
        .await?
        .ok_or(Error::CurrentEncounterDoesNotExist)?;

    if !encounter.character_ids.contains(&body.character_id) {
        return Err(Error::CharacterNotInEncounter(body.character_id));
    }

    let now = Utc::now();
    let operation = Operation {
        id: OperationId::new(),
        campaign_id: campaign_id,
        encounter_id: Some(encounter.id),
        character_id: body.character_id,
        created_at: now,
        modified_at: now,
        operation_type: OperationType::Move { feet: body.feet },
    };

    db::insert_operation(&db, &operation).await?;

    Ok(Json(OperationBody::render(operation)))
}
