use std::fmt::{Debug, Display};

use actix_web::body::Body;
use actix_web::error::{JsonPayloadError, PathError, QueryPayloadError, UrlencodedError};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use mongodb::bson::ser::Error as BsonError;
use mongodb::error::Error as DatabaseError;
use serde::{Serialize, Serializer};

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::encounter::EncounterId;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Error {
    // 400
    #[serde(serialize_with = "display")]
    InvalidJson(JsonPayloadError),
    #[serde(serialize_with = "display")]
    InvalidPath(PathError),
    #[serde(serialize_with = "display")]
    InvalidForm(UrlencodedError),
    #[serde(serialize_with = "display")]
    InvalidQuery(QueryPayloadError),

    // 404
    PathDoesNotExist,
    CampaignDoesNotExist {
        campaign_id: CampaignId,
    },
    CharacterDoesNotExistInCampaign {
        campaign_id: CampaignId,
        character_id: CharacterId,
    },
    CurrentEncounterDoesNotExist {
        campaign_id: CampaignId,
    },

    // 409
    ConcurrentModificationDetected,
    CurrentEncounterAlreadyExists {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
    },
    CharacterNotInCampaign {
        campaign_id: CampaignId,
        character_id: CharacterId,
    },
    CharacterNotInEncounter {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
        character_id: CharacterId,
    },

    // 500
    #[serde(serialize_with = "display")]
    FailedDatabaseCall(DatabaseError),
    #[serde(serialize_with = "display")]
    FailedToSerializeToBson(BsonError),
}

impl Error {
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::InvalidJson(_) => "E4001000",
            Error::InvalidPath(_) => "E4001001",
            Error::InvalidForm(_) => "E4001002",
            Error::InvalidQuery(_) => "E4001003",
            Error::PathDoesNotExist => "E4041000",
            Error::CampaignDoesNotExist { .. } => "E4041001",
            Error::CharacterDoesNotExistInCampaign { .. } => "E4041002",
            Error::CurrentEncounterDoesNotExist { .. } => "E4041003",
            Error::ConcurrentModificationDetected => "E4091000",
            Error::CurrentEncounterAlreadyExists { .. } => "E4091001",
            Error::CharacterNotInCampaign { .. } => "E4091002",
            Error::CharacterNotInEncounter { .. } => "E4091003",
            Error::FailedDatabaseCall { .. } => "E5001000",
            Error::FailedToSerializeToBson { .. } => "E5001001",
        }
    }

    pub fn error_message(&self) -> &'static str {
        match self {
            Error::InvalidJson(_) => "The given json could not be parsed",
            Error::InvalidPath(_) => "The given path could not be parsed",
            Error::InvalidForm(_) => "The given form could not be parsed",
            Error::InvalidQuery(_) => "The given query could not be parsed",
            Error::PathDoesNotExist => "The requested path does not exist",
            Error::CampaignDoesNotExist { .. } => "The requested campaign does not exist",
            Error::CharacterDoesNotExistInCampaign { .. } => {
                "The requested character is not in the campaign"
            }
            Error::CurrentEncounterDoesNotExist { .. } => {
                "The requested campaign is not currently in an encounter"
            }
            Error::ConcurrentModificationDetected => {
                "The server detected a concurrent modification"
            }
            Error::CurrentEncounterAlreadyExists { .. } => {
                "The requested campaign is currently in an encounter"
            }
            Error::CharacterNotInCampaign { .. } => {
                "The requested operation uses a character that is not in the campaign"
            }
            Error::CharacterNotInEncounter { .. } => {
                "The requested operation uses a character that is not in the encounter"
            }
            Error::FailedDatabaseCall { .. } => {
                "An error occurred when communicating with the database"
            }
            Error::FailedToSerializeToBson { .. } => {
                "An error occurred when serializing an object to bson"
            }
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::InvalidJson(_) => StatusCode::BAD_REQUEST,
            Error::InvalidPath(_) => StatusCode::BAD_REQUEST,
            Error::InvalidForm(_) => StatusCode::BAD_REQUEST,
            Error::InvalidQuery(_) => StatusCode::BAD_REQUEST,
            Error::PathDoesNotExist => StatusCode::NOT_FOUND,
            Error::CampaignDoesNotExist { .. } => StatusCode::NOT_FOUND,
            Error::CharacterDoesNotExistInCampaign { .. } => StatusCode::NOT_FOUND,
            Error::CurrentEncounterDoesNotExist { .. } => StatusCode::NOT_FOUND,
            Error::ConcurrentModificationDetected => StatusCode::CONFLICT,
            Error::CurrentEncounterAlreadyExists { .. } => StatusCode::CONFLICT,
            Error::CharacterNotInCampaign { .. } => StatusCode::CONFLICT,
            Error::CharacterNotInEncounter { .. } => StatusCode::CONFLICT,
            Error::FailedDatabaseCall(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FailedToSerializeToBson(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<Body> {
        #[derive(Serialize)]
        struct Dummy<'a> {
            error_code: &'static str,
            error_message: &'static str,
            error_meta: &'a Error,
        }

        HttpResponse::build(self.status_code()).json(&Dummy {
            error_code: self.error_code(),
            error_message: self.error_message(),
            error_meta: self,
        })
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Error {
        Error::FailedDatabaseCall(error)
    }
}

impl From<BsonError> for Error {
    fn from(error: BsonError) -> Error {
        Error::FailedToSerializeToBson(error)
    }
}

fn display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(value)
}
