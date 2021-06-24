use std::fmt::{Debug, Display};
use std::io::Error as IoError;

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
use crate::item::ItemId;

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
    ItemDoesNotExist {
        item_id: ItemId,
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
    CharacterAlreadyRolledInitiative {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
        character_id: CharacterId,
    },
    CharactersHaveNotRolledInitiative {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
        character_ids: Vec<CharacterId>,
    },
    NoCharactersInEncounter {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
    },
    NotThisPlayersTurn {
        campaign_id: CampaignId,
        encounter_id: EncounterId,
        request_character_id: CharacterId,
        current_character_id: CharacterId,
    },
    ItemIsNotAWeapon {
        item_id: ItemId,
    },
    CharacterDoesNotHavePosition {
        character_id: CharacterId,
    },
    CharacterMovementExceeded {
        character_id: CharacterId,
        maximum_movement: f32,
        current_movement: f32,
        request_movement: f32,
    },
    AttackNotInRange {
        request_character_id: CharacterId,
        target_character_id: CharacterId,
        weapon_range: f32,
        current_range: f32,
    },

    // 500
    #[serde(serialize_with = "display")]
    FailedDatabaseCall(DatabaseError),
    #[serde(serialize_with = "display")]
    FailedToSerializeToBson(BsonError),
    #[serde(serialize_with = "display")]
    IoError(IoError),
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
            Error::ItemDoesNotExist { .. } => "E4041004",
            Error::ConcurrentModificationDetected => "E4091000",
            Error::CurrentEncounterAlreadyExists { .. } => "E4091001",
            Error::CharacterNotInCampaign { .. } => "E4091002",
            Error::CharacterNotInEncounter { .. } => "E4091003",
            Error::CharacterAlreadyRolledInitiative { .. } => "E4091004",
            Error::CharactersHaveNotRolledInitiative { .. } => "E4091005",
            Error::NoCharactersInEncounter { .. } => "E4091006",
            Error::NotThisPlayersTurn { .. } => "E4091007",
            Error::ItemIsNotAWeapon { .. } => "E4091008",
            Error::CharacterDoesNotHavePosition { .. } => "E4091009",
            Error::CharacterMovementExceeded { .. } => "E4091010",
            Error::AttackNotInRange { .. } => "E4091011",
            Error::FailedDatabaseCall(_) => "E5001000",
            Error::FailedToSerializeToBson(_) => "E5001001",
            Error::IoError(_) => "E5001002",
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
            Error::ItemDoesNotExist { .. } => "The requested item does not exist",
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
            Error::CharacterAlreadyRolledInitiative { .. } => {
                "The requested character has already rolled initiative"
            }
            Error::CharactersHaveNotRolledInitiative { .. } => {
                "The requested encounter has characters that have not rolled initiative"
            }
            Error::NoCharactersInEncounter { .. } => {
                "The requested encounter has no characters to start with"
            }
            Error::NotThisPlayersTurn { .. } => {
                "The requested player does not have permission for this turn"
            }
            Error::ItemIsNotAWeapon { .. } => "The provided item was expected to be a weapon",
            Error::CharacterDoesNotHavePosition { .. } => {
                "The requested character does not have a position"
            }
            Error::CharacterMovementExceeded { .. } => {
                "The requested character does not have enough speed to make this move"
            }
            Error::AttackNotInRange { .. } => {
                "The requested attack does not have enough range to reach the target character"
            }
            Error::FailedDatabaseCall { .. } => {
                "An error occurred when communicating with the database"
            }
            Error::FailedToSerializeToBson { .. } => {
                "An error occurred when serializing an object to bson"
            }
            Error::IoError { .. } => "An error occurred during an I/O operation",
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
            Error::ItemDoesNotExist { .. } => StatusCode::NOT_FOUND,
            Error::ConcurrentModificationDetected => StatusCode::CONFLICT,
            Error::CurrentEncounterAlreadyExists { .. } => StatusCode::CONFLICT,
            Error::CharacterNotInCampaign { .. } => StatusCode::CONFLICT,
            Error::CharacterNotInEncounter { .. } => StatusCode::CONFLICT,
            Error::CharacterAlreadyRolledInitiative { .. } => StatusCode::CONFLICT,
            Error::CharactersHaveNotRolledInitiative { .. } => StatusCode::CONFLICT,
            Error::NoCharactersInEncounter { .. } => StatusCode::CONFLICT,
            Error::NotThisPlayersTurn { .. } => StatusCode::CONFLICT,
            Error::ItemIsNotAWeapon { .. } => StatusCode::CONFLICT,
            Error::CharacterDoesNotHavePosition { .. } => StatusCode::CONFLICT,
            Error::CharacterMovementExceeded { .. } => StatusCode::CONFLICT,
            Error::AttackNotInRange { .. } => StatusCode::CONFLICT,
            Error::FailedDatabaseCall(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FailedToSerializeToBson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::IoError(error)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidJson(err) => Some(err),
            Error::InvalidPath(err) => Some(err),
            Error::InvalidForm(err) => Some(err),
            Error::InvalidQuery(err) => Some(err),
            Error::FailedDatabaseCall(err) => Some(err),
            Error::FailedToSerializeToBson(err) => Some(err),
            _ => None,
        }
    }
}

fn display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(value)
}
