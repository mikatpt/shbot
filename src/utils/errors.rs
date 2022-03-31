use axum::{http::StatusCode, response::IntoResponse, Json};
use color_eyre::eyre;
use serde_json::json;
use tracing::{error, warn};

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
    #[error("Unknown error")]
    Unknown,
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
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
}

// Directly pass through all application error types to report to user.
// We do not report the various specific application errors
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

/// We log all errors during the `IntoResponse` conversion.
///
/// Note the distinction between `logging` and `reporting`: Reports are specifically for the end
/// user, and disguise internal errors; logs are for us, and hide nothing.
///
/// User errors are logged at `warn` and application errors are logged at `error`.
fn log_error(err: &UserError) {
    type E = UserError;
    match err {
        E::InvalidArg(_) | E::Duplicate(_) | E::NotFound(_) => warn!("{}", err),
        _ => error!("{}", err),
    }
}

// Slack requires that we always send 200's when reporting errors.
impl IntoResponse for UserError {
    fn into_response(self) -> axum::response::Response {
        log_error(&self);
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
