use axum::{
    body::Bytes,
    extract::{Extension, Form, Path},
    http::StatusCode,
    response::Html,
    Json,
};
use futures::stream::FuturesUnordered;
use serde_json::Value;
use tracing::{debug, info};

use crate::{
    models::{Film, Priority},
    server::{Result, State},
    slack::slash::{ResponseType, SlashRequest, SlashResponse},
    slack::{
        self,
        events::{EventRequest, EventType},
    },
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

// --------------- Event API --------------- //

pub(super) async fn event_api_entrypoint(
    Json(request): Json<Value>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, String)> {
    debug!("handling event: {}", request);

    if let Some(r) = request.get("type") {
        if Some("url_verification") == r.as_str() {
            return Ok((StatusCode::OK, slack::auth_challenge(request)?));
        }
    }

    let request: EventRequest = serde_json::from_value(request).map_err(Into::<Error>::into)?;

    tokio::spawn(request.handle_event(state));

    Ok((StatusCode::OK, "".to_string()))
}

// --------------- Films Handlers --------------- //

pub(super) async fn list_films(Extension(state): Extension<State>) -> Result<Json<Vec<Film>>> {
    info!("Retrieving films...");

    match state.db.list_films().await {
        Ok(films) => Ok(Json(films)),
        Err(e) => Err(e.into()),
    }
}

pub(super) async fn get_film(
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Film>> {
    info!("Retrieving film {name}");

    match state.db.get_film(&name).await {
        Ok(Some(film)) => Ok(Json(film)),
        Ok(None) => Err(UserError::NotFound("do stuff".to_string())),
        Err(e) => Err(e.into()),
    }
}

pub(super) async fn insert_films(
    form: Form<SlashRequest>,
    Extension(state): Extension<State>,
) -> Result<Json<SlashResponse>> {
    let slash_command = form.0;

    let (priority, films) = slash_command
        .text
        .trim()
        .split_once(' ')
        .unwrap_or_default();

    info!("Inserting {priority} prio films:");
    info!("{films:?}");

    let priority: Priority = if let Ok(p) = priority.parse() {
        p
    } else {
        let mut msg = "I wasn't able to read your command :(\n".to_string();
        msg += "Command format is: /insertfilms HIGH film, film, film...\n\n";
        msg += &format!("Your command was /insertfilms {}.\n", slash_command.text);
        let res = SlashResponse::new(msg, Some(ResponseType::Ephemeral));
        return Ok(Json(res));
    };

    // Concurrently insert all films
    let films: FuturesUnordered<_> = films
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|film| {
            let s = state.clone();
            tokio::spawn(async move {
                info!("inserting {}", &film);
                s.db.insert_film(&film, priority).await
            })
        })
        .collect();

    // Await all inserts to complete.
    let results = futures::future::join_all(films).await;

    let (mut success, mut fail) = (0, 0);
    results.iter().for_each(|r| {
        let incr = if let Ok(Err(_)) = r { (0, 1) } else { (1, 0) };
        success += incr.0;
        fail += incr.1;
    });

    // Write user response message.
    let mut msg = String::new();
    if success > 0 {
        msg += &format!("Successfully inserted {success} film(s):\n");
    }

    for res in results.iter().flatten().flatten() {
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
        let res = res.map_err(Into::<Error>::into)?;
        if let Err(e) = res {
            msg += "\n";
            msg += &e.to_string();
        }
    }
    let response = SlashResponse::new(msg, Some(ResponseType::Ephemeral));

    Ok(Json(response))
}
