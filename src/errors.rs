use axum::{http::StatusCode, response::IntoResponse, Json};
use color_eyre::eyre;
use serde_json::json;
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // User errors (expected)
    #[error("Invalid input")]
    InvalidArg(String),
    #[error("Duplicate data: {0}")]
    Duplicate(String),
    #[error("Not found")]
    NotFound,

    // Application errors (unexpected)
    #[error("Unknown error")]
    Unknown,
    #[error("Internal error: {0}")]
    Internal(#[from] eyre::Error),
    #[error("Repository error")]
    Repository(#[from] tokio_postgres::Error),
    #[error("Internal error: {0}")]
    Server(#[from] axum::Error),
    #[error("Internal error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("Internal error: {0}")]
    CreatePool(#[from] deadpool_postgres::CreatePoolError),
}

/// User errors logged at `warn`.
/// Application errors logged at `error`.
fn report_error(err: &Error) {
    match err {
        Error::InvalidArg(_) | Error::Duplicate(_) | Error::NotFound => warn!("{}", err),
        _ => error!("{}", err),
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        report_error(&self);
        let (status, error_msg) = match self {
            Error::InvalidArg(_) | Error::Duplicate(_) => {
                (StatusCode::BAD_REQUEST, format!("{}", self))
            }
            Error::NotFound => (StatusCode::NOT_FOUND, format!("{}", self)),
            Error::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_msg,
        }));

        (status, body).into_response()
    }
}
