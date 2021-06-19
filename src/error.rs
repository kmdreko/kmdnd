use std::fmt::{Debug, Display};

use actix_web::body::Body;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use mongodb::bson::ser::Error as BsonError;
use mongodb::error::Error as DatabaseError;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json::Value;

use crate::campaign::CampaignId;
use crate::character::CharacterId;
use crate::encounter::EncounterId;

#[derive(Debug)]
pub enum Error {
    // 404
    CampaignDoesNotExist(CampaignId),
    CharacterDoesNotExist(CharacterId),
    CurrentEncounterDoesNotExist,

    // 409
    CurrentEncounterAlreadyExists(EncounterId),
    CharacterNotInCampaign(CharacterId),
    CharacterNotInEncounter(CharacterId),

    // 500
    FailedDatabaseCall(DatabaseError),
    FailedToSerializeToBson(BsonError),
}

impl Error {
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::CampaignDoesNotExist(_) => "E4041000",
            Error::CharacterDoesNotExist(_) => "E4041001",
            Error::CurrentEncounterDoesNotExist => "E4041002",
            Error::CurrentEncounterAlreadyExists(_) => "E4091000",
            Error::CharacterNotInCampaign(_) => "E4091001",
            Error::CharacterNotInEncounter(_) => "E4091002",
            Error::FailedDatabaseCall(_) => "E5001000",
            Error::FailedToSerializeToBson(_) => "E5001001",
        }
    }

    pub fn error_message(&self) -> &'static str {
        match self {
            Error::CampaignDoesNotExist(_) => "The requested campaign does not exist",
            Error::CharacterDoesNotExist(_) => "The requested character does not exist",
            Error::CurrentEncounterDoesNotExist => {
                "The requested campaign is not currently in an encounter"
            }
            Error::CurrentEncounterAlreadyExists(_) => {
                "The requested campaign is currently in an encounter"
            }
            Error::CharacterNotInCampaign(_) => {
                "The requested operation uses a character that is not in the campaign"
            }
            Error::CharacterNotInEncounter(_) => {
                "The requested operation uses a character that is not in the encounter"
            }
            Error::FailedDatabaseCall(_) => {
                "An error occurred when communicating with the database"
            }
            Error::FailedToSerializeToBson(_) => {
                "An error occurred when serializing an object to bson"
            }
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::CampaignDoesNotExist(_) => StatusCode::NOT_FOUND,
            Error::CharacterDoesNotExist(_) => StatusCode::NOT_FOUND,
            Error::CurrentEncounterDoesNotExist => StatusCode::NOT_FOUND,
            Error::CurrentEncounterAlreadyExists(_) => StatusCode::CONFLICT,
            Error::CharacterNotInCampaign(_) => StatusCode::CONFLICT,
            Error::CharacterNotInEncounter(_) => StatusCode::CONFLICT,
            Error::FailedDatabaseCall(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FailedToSerializeToBson(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<Body> {
        HttpResponse::build(self.status_code()).json(&self)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 3)?;
        state.serialize_field("error_code", &self.error_code())?;
        state.serialize_field("error_message", &self.error_message())?;

        match self {
            Error::CampaignDoesNotExist(campaign_id) => {
                state.serialize_field("error_meta", campaign_id)?
            }
            Error::CharacterDoesNotExist(character_id) => {
                state.serialize_field("error_meta", character_id)?
            }
            Error::CurrentEncounterDoesNotExist => {
                state.serialize_field("error_meta", &Value::Null)?
            }
            Error::CurrentEncounterAlreadyExists(encounter_id) => {
                state.serialize_field("error_meta", encounter_id)?
            }
            Error::CharacterNotInCampaign(character_id) => {
                state.serialize_field("error_meta", character_id)?
            }
            Error::CharacterNotInEncounter(character_id) => {
                state.serialize_field("error_meta", character_id)?
            }
            Error::FailedDatabaseCall(db_error) => {
                state.serialize_field("error_meta", &db_error.to_string())?
            }
            Error::FailedToSerializeToBson(bson_error) => {
                state.serialize_field("error_meta", &bson_error.to_string())?
            }
        };

        state.end()
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
