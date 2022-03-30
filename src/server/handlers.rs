use axum::{
    extract::{Extension, Form, Path},
    response::Html,
    Json,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    models::{slack::SlackSlashCommand, Film},
    server::State,
    Error, Result,
};

pub(super) async fn home() -> Html<&'static str> {
    Html("<h1>Hello from the ShereeBot server!</h1>")
}

pub(super) async fn list_films(Extension(state): Extension<State>) -> Result<Json<Vec<Film>>> {
    info!("Retrieving films...");
    match state.db.list_films().await {
        Ok(films) => Ok(Json(films)),
        Err(e) => Err(e),
    }
}

pub(super) async fn get_film(
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<Film>> {
    info!("Retrieving film {name}");

    match state.db.get_film(&name).await {
        Ok(Some(film)) => Ok(Json(film)),
        Ok(None) => Err(Error::NotFound("do stuff")),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct InsertFilmRequest {
    name: String,
}

pub(super) async fn insert_film(
    Json(film_req): Json<InsertFilmRequest>,
    Extension(state): Extension<State>,
) -> Result<Json<Film>> {
    info!("inserting {}", film_req.name);

    match state.db.insert_film(&film_req.name).await {
        Ok(film) => Ok(Json(film)),
        Err(e) => Err(e),
    }
}

//text: "film1, film2, film 3,film 4 ,",
pub(super) async fn testing(Form(slash_command): Form<SlackSlashCommand>) -> Json<&'static str> {
    dbg!(slash_command);
    Json("{}")
}
