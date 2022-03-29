use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::{body::Bytes, response::Response, Router};

use tower::{timeout::TimeoutLayer, BoxError, ServiceBuilder};
use tower_http::{
    classify::ServerErrorsFailureClass, compression::CompressionLayer, trace::TraceLayer,
};
use tracing::{debug, Span};

/// Attaches interceptors to router.
pub(crate) fn attach(router: Router) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::REQUEST_TIMEOUT
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    tracing::debug_span!(
                        "request",
                        req = %request.method(),
                        uri = %request.uri().path(),
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {})
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    match latency.as_micros() {
                        0..=999 => debug!(
                            "[{}] Served in {}μs",
                            response.status(),
                            latency.as_micros()
                        ),
                        1000..=999999 => debug!(
                            "[{}] Served in {}ms",
                            response.status(),
                            latency.as_millis()
                        ),
                        _ => debug!("[{}] Served in {}s", response.status(), latency.as_secs()),
                    };
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    // ..
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        // ...
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // ...
                    },
                ),
        );

    router.layer(middleware)
}
