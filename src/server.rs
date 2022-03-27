use std::sync::Arc;

use axum::Json;
use axum::http::StatusCode;
use axum::{Router, extract::Extension, routing::get};
use axum::response::{Html, IntoResponse};
use color_eyre::Result;
use tracing::info;


use crate::config::Config;

struct State {test: i32}

pub async fn serve(cfg: &Config) -> Result<()> {
    let postgres_pool = crate::db::create_pool(&cfg.postgres);
    let state = Arc::new(State{test: 1});
    let app = Router::new()
        .route("/", get(home).post(create))
        .route("/_health", get(health_check))
        .layer(Extension(state));

    info!("Serving shereebot at http://localhost:{}", &cfg.server.address);
    axum::Server::bind(&cfg.server.address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn home(Extension(state): Extension<Arc<State>>) -> Html<&'static str> {
    info!("message! {}", state.test);

    Html("<h1>Hello from the ShereeBot server!</h1>")
}

async fn create(
    Json(input): Json<serde_json::Value>,
    Extension(state): Extension<Arc<State>>
) -> impl IntoResponse {
    if let Ok(j) = serde_json::to_string(&input) {
        info!("j is {}", j);
    }

    StatusCode::OK

}


async fn health_check() -> Json<&'static str> {
    Json("All healthy!")
}
