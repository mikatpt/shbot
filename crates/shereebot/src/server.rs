use std::sync::Arc;

use axum::{
    extract::Extension,
    response::Html,
    routing::{get, post},
    Router,
};
use tracing::info;

use crate::{config::Config, queue::Queue, store::Database, UserError};
mod handlers;
mod interceptors;

/// Contains all server-wide stateful data.
pub(crate) type State = Arc<InnerState>;

/// All server results must return a UserError.
/// This allows us to report readable errors and hide internal errors.
pub type Result<T> = std::result::Result<T, UserError>;

pub(crate) struct InnerState {
    pub(crate) db: Database,
    pub(crate) oauth_token: String,
    pub(crate) req_client: reqwest::Client,
    pub(crate) queue: Queue,
}

impl InnerState {
    pub(crate) fn _new() -> State {
        let v = reqwest::tls::Version::TLS_1_2;
        let req_client = reqwest::Client::builder()
            .min_tls_version(v)
            .build()
            .unwrap_or_default();
        Arc::new(Self {
            db: crate::store::new_mock(),
            oauth_token: "".to_string(),
            queue: Queue::_new(),
            req_client,
        })
    }
}

impl std::fmt::Debug for InnerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("<State>").finish()
    }
}

async fn initialize_state(cfg: &Config) -> color_eyre::Result<State> {
    let db = crate::store::new(&cfg.postgres)?;
    let oauth_token = cfg.token.to_string();
    let v = reqwest::tls::Version::TLS_1_2;
    let req_client = reqwest::Client::builder()
        .use_rustls_tls()
        .min_tls_version(v)
        .build()?;
    let queue = Queue::from_db(db.clone()).await?;

    let state = InnerState {
        db,
        oauth_token,
        req_client,
        queue,
    };

    Ok(Arc::new(state))
}

/// Initializes server state and runs the server.
pub async fn serve(cfg: &Config) -> color_eyre::Result<()> {
    let state = initialize_state(cfg).await?;

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
            get(handlers::list_films),
            // .post(handlers::insert_films::<T>),
        )
        .route("/events", post(handlers::events_api_entrypoint))
        .route("/_health", get(health_check))
        .route("/testing", post(handlers::testing))
        .layer(Extension(state));

    interceptors::attach(app)
}

async fn health_check() -> Html<&'static str> {
    Html("<div>Hello from ShereeBot's health checkpoint</div>")
}
