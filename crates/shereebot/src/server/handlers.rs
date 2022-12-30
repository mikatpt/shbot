use axum::{body::Bytes, extract::Extension, http::StatusCode, response::Html, Json};
use serde_json::Value;
use tracing::{debug, error, info, trace};

use crate::{
    server::{Result, State},
    slack::events::EventRequest,
    slack::slash::{ResponseType, SlashResponse},
    Error,
};
use models::Film;

/// Just for testing poorly documented slack endpoints.
pub(super) async fn testing(body: Bytes) -> Result<Json<SlashResponse>> {
    debug!("{:?}", body);

    // Do some processing
    let res = SlashResponse::new("testing".to_string(), Some(ResponseType::Ephemeral));

    // Return a payload for the app to parse.
    Ok(Json(res))
}

pub(super) async fn home() -> Html<&'static str> {
    Html("<h1>Hello from the ShereeBot server!</h1>")
}

// --------------- Events API --------------- //

pub(super) async fn events_api_entrypoint(
    Json(request): Json<Value>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, String)> {
    if let Some(challenge) = request.get("challenge") {
        info!("Auth challenge received");
        return Ok((StatusCode::OK, challenge.to_string()));
    }
    trace!("req: {:?}", request.as_object());

    let request: EventRequest = serde_json::from_value(request).map_err(|e| -> Error {
        error!("{e}");
        e.into()
    })?;

    // ignore bot events to avoid infinite loop.
    if request.authorizations[0].is_bot {
        return Ok((StatusCode::OK, "".to_string()));
    }

    tokio::spawn(request.handle_event(state));

    Ok((StatusCode::OK, "".to_string()))
}

// --------------- Films Handlers --------------- //

#[tracing::instrument]
pub(super) async fn list_films(Extension(state): Extension<State>) -> Result<Json<Vec<Film>>> {
    info!("Retrieving films...");

    match state.db.list_films().await {
        Ok(films) => Ok(Json(films)),
        Err(e) => {
            error!("{e}");
            Err(e.into())
        }
    }
}

// #[tracing::instrument(skip_all)]
// pub(super) async fn insert_films<T: Client>(
//     form: Form<SlashRequest>,
//     Extension(state): Extension<State<T>>,
// ) -> Result<Json<SlashResponse>> {
//     let slash_command = form.0;

//     let manager = FilmManager::new(state);
//     let msg = manager.insert_films(&slash_command.text).await;
//     let res = SlashResponse::new(msg, Some(ResponseType::Ephemeral));

//     Ok(Json(res))
// }
