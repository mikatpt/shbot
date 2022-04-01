use axum::{
    body::Bytes,
    extract::{Extension, Form},
    http::StatusCode,
    response::Html,
    Json,
};
use serde_json::Value;
use tracing::{debug, error, info};

use crate::{
    films::FilmManager,
    models::Film,
    server::{Result, State},
    slack::events::EventRequest,
    slack::slash::{ResponseType, SlashRequest, SlashResponse},
    Error, UserError,
};

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

#[tracing::instrument(skip_all)]
pub(super) async fn events_api_entrypoint(
    Json(request): Json<Value>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, String)> {
    debug!("handling event: {}", request);

    if let Some(challenge) = request.get("challenge") {
        info!("Auth challenge received");
        return Ok((StatusCode::OK, challenge.to_string()));
    }

    let request: EventRequest = serde_json::from_value(request).map_err(|e| -> Error {
        error!("{e}");
        e.into()
    })?;

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

#[tracing::instrument(skip_all)]
pub(super) async fn insert_films(
    form: Form<SlashRequest>,
    Extension(state): Extension<State>,
) -> Result<Json<SlashResponse>> {
    let slash_command = form.0;

    let manager = FilmManager::new(state);
    let results = match manager.insert_films(&slash_command.text).await {
        Ok(r) => r,
        Err(e) => {
            let err = UserError::from(e);
            let res = SlashResponse::new(format!("{}", err), Some(ResponseType::Ephemeral));
            return Ok(Json(res));
        }
    };

    let (mut success, mut fail) = (0, 0);
    results.iter().for_each(|r| {
        let incr = if r.is_err() { (0, 1) } else { (1, 0) };
        success += incr.0;
        fail += incr.1;
    });

    // Write user response message.
    let mut msg = String::new();
    if success > 0 {
        msg += &format!("Successfully inserted {success} film(s):\n");
    }

    for res in results.iter().flatten() {
        msg += &format!("{}, ", res.name);
    }
    if success > 0 {
        msg.pop();
        msg.pop();
    }

    if success > 0 && fail > 0 {
        msg += "\n\n";
    }
    if fail > 0 {
        msg += "Some films were not inserted:\n";
    }

    for res in results {
        if let Err(e) = res {
            msg += "\n";
            msg += &e.to_string();
        }
    }
    let response = SlashResponse::new(msg, Some(ResponseType::Ephemeral));

    Ok(Json(response))
}
