use std::sync::Arc;

use axum::{
    extract::Extension,
    response::Html,
    routing::{get, post},
    Router,
};
use tracing::info;

use crate::{
    config::Config,
    store::{Database, PostgresClient},
    UserError,
};
mod handlers;
mod interceptors;

/// Contains all server-wide stateful data.
type State = Arc<InnerState>;

/// All server results must return a UserError.
/// This allows us to report readable errors and hide internal errors.
type Result<T> = std::result::Result<T, UserError>;

#[derive(Debug)]
struct InnerState {
    db: Database<PostgresClient>,
}

impl InnerState {
    fn new(db: Database<PostgresClient>) -> State {
        Arc::new(InnerState { db })
    }
}

/// Initializes server state and runs the server.
pub async fn serve(cfg: &Config) -> color_eyre::Result<()> {
    let db = crate::store::new(&cfg.postgres)?;
    let state = InnerState::new(db);

    let app = new_router(state);

    info!("Serving shereebot at http://localhost:{}", cfg.server.port);

    axum::Server::bind(&cfg.server.address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Initialize axum app and attach all routes.
fn new_router(state: State) -> Router {
    let app = Router::new()
        .route("/", get(handlers::home))
        .route(
            "/films",
            get(handlers::list_films).post(handlers::insert_films),
        )
        .route("/films/:name", get(handlers::get_film))
        .route("/_challenge", post(handlers::auth_challenge))
        .route("/_health", get(health_check))
        .route("/testing", post(handlers::testing))
        .layer(Extension(state));

    interceptors::attach(app)
}

async fn health_check() -> Html<&'static str> {
    Html("<div>Hello from ShereeBot's health checkpoint</div>")
}
