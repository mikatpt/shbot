use axum::{http::StatusCode, response::IntoResponse, Json};
use color_eyre::eyre;
use serde_json::json;

/// All possible application errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // User errors (expected)
    #[error("Invalid input: {0}")]
    InvalidArg(String),
    #[error("Duplicate data: {0}")]
    Duplicate(String),
    #[error("Not found: {0}")]
    NotFound(String),

    // Application errors (unexpected)
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error(transparent)]
    Internal(#[from] eyre::Error),
    #[error(transparent)]
    Repository(#[from] tokio_postgres::Error),
    #[error(transparent)]
    Server(#[from] axum::Error),
    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error(transparent)]
    CreatePool(#[from] deadpool_postgres::CreatePoolError),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ParseError(#[from] strum::ParseError),
    #[error(transparent)]
    JsonError(#[from] serde_json::error::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    EnvVarError(#[from] std::env::VarError),
    #[error("Unreachable.")]
    Unreachable,
}

/// All error types reported to the end user.
#[derive(thiserror::Error, Debug)]
pub enum UserError {
    // Expected errors
    #[error("Invalid input: {0}")]
    InvalidArg(String),
    #[error("Duplicate data: {0}")]
    Duplicate(String),
    #[error("Not found: {0}")]
    NotFound(String),

    // Unexpected errors
    #[error("Internal error: {0}")]
    Internal(Error),
}

// Directly pass through all application error types to report to user.
// We log, but do not report, the various internal application errors.
impl From<Error> for UserError {
    fn from(e: Error) -> Self {
        type E = Error;
        match e {
            E::InvalidArg(s) => Self::InvalidArg(s),
            E::Duplicate(s) => Self::Duplicate(s),
            E::NotFound(s) => Self::NotFound(s),
            _ => Self::Internal(e),
        }
    }
}

// Slack requires that we always send 200's when reporting errors.
impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        type E = UserError;
        let error_msg = match self {
            E::InvalidArg(_) | E::Duplicate(_) | E::NotFound(_) => {
                format!("{}", self)
            }
            _ => "Internal Error! Please let Michael know.".to_string(),
        };

        // TODO: Change into slack response
        let body = Json(json!({
            "error": error_msg,
        }));

        (StatusCode::OK, body).into_response()
    }
}
