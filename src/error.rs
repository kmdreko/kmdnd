use std::fmt::{Debug, Display};

use actix_web::body::Body;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use mongodb::bson::ser::Error as BsonError;
use mongodb::error::Error as DatabaseError;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    // 404
    CampaignDoesNotExist(String),
    CharacterDoesNotExist(String),

    // 500
    FailedDatabaseCall(DatabaseError),
    FailedToSerializeToBson(BsonError),
}

impl Error {
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::CampaignDoesNotExist(_) => "E4041000",
            Error::CharacterDoesNotExist(_) => "E4041000",
            Error::FailedDatabaseCall(_) => "E5001000",
            Error::FailedToSerializeToBson(_) => "E5001001",
        }
    }

    pub fn error_message(&self) -> &'static str {
        match self {
            Error::CampaignDoesNotExist(_) => "The requested campaign does not exist",
            Error::CharacterDoesNotExist(_) => "The requested character does not exist",
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
            Error::FailedDatabaseCall(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::FailedToSerializeToBson(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<Body> {
        #[derive(Serialize)]
        struct ErrorResponseBody {
            error_code: &'static str,
            error_message: &'static str,
            // error_meta: &'a Serialize,
        }

        let body = ErrorResponseBody {
            error_code: self.error_code(),
            error_message: self.error_message(),
        };

        HttpResponse::build(self.status_code()).json(body)
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
