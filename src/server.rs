use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{extract::Extension, routing::get, Json, Router};

use color_eyre::Result;

use tracing::info;

use crate::config::Config;
use crate::interceptors;

#[derive(Debug)]
struct State {
    test: i32,
}

pub async fn serve(cfg: &Config) -> Result<()> {
    let postgres_pool = crate::db::create_pool(&cfg.postgres);
    let address = &cfg.server.address;
    let state = Arc::new(State { test: 1 });

    let app = router(state);

    info!("Serving shereebot at http://localhost:{}", address);
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn router(state: Arc<State>) -> Router {
    let app = Router::new()
        .route("/", get(home).post(create))
        .route("/_health", get(health_check))
        .layer(Extension(state));

    interceptors::attach(app)
}

async fn home(Extension(state): Extension<Arc<State>>) -> Html<&'static str> {
    info!("message! {}", state.test);

    std::thread::sleep(Duration::from_secs(2));

    info!("did some processing");

    Html("<h1>Hello from the ShereeBot server!</h1>")
}

async fn create(
    Json(input): Json<serde_json::Value>,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    if let Ok(j) = serde_json::to_string(&input) {
        info!("j is {}", j);
    }

    StatusCode::OK
}

async fn health_check() -> Html<&'static str> {
    Html("<div>Hello from ShereeBot's health checkpoint</div>")
}
