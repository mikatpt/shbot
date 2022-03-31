use axum::{
    extract::{Extension, Form, Path},
    response::Html,
    Json,
};
use futures::stream::FuturesUnordered;

use tracing::info;

use crate::{
    models::slack::{AppMention, AuthChallenge, ResponseType, SlashRequest, SlashResponse},
    models::{Film, Priority},
    server::{Result, State},
    UserError,
};

pub(super) async fn testing(Json(body): Json<AppMention>) -> Result<Json<String>> {
    // Do some processing

    // Return a payload for the app to parse.
    Ok(Json("hey".into()))
}

pub(super) async fn home() -> Html<&'static str> {
    Html("<h1>Hello from the ShereeBot server!</h1>")
}

pub(super) async fn auth_challenge(Json(body): Json<AuthChallenge>) -> Result<String> {
    if body.r#type != "url_verification" {
        return Err(UserError::InvalidArg("Not an auth challenge".into()));
    }
    Ok(body.challenge)
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
    Form(slash_command): Form<SlashRequest>,
    Extension(state): Extension<State>,
) -> Result<Json<SlashResponse>> {
    let (priority, films) = slash_command
        .text
        .trim()
        .split_once(' ')
        .unwrap_or_default();

    let priority: Priority = if let Ok(p) = priority.to_uppercase().parse() {
        p
    } else {
        let mut msg = "I wasn't able to read your command :(\n".to_string();
        msg += "Command format is: /insertfilms HIGH film, film, film.\n\n";
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
        msg += &format!("Successfully inserted {success} film(s)!");
    }
    if success > 0 && fail > 0 {
        msg += "\n";
    }
    if fail > 0 {
        msg += "Some films were not inserted:";
    }

    for res in results {
        let res = res?;
        if let Err(e) = res {
            msg += "\n";
            msg += &e.to_string();
        }
    }
    let response = SlashResponse::new(msg, Some(ResponseType::Ephemeral));

    Ok(Json(response))
}
