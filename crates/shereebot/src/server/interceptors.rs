use std::time::Duration;

use axum::{
    body::Bytes,
    error_handling::HandleErrorLayer,
    http::{HeaderMap, Request, StatusCode},
    response::Response,
    Router,
};
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
                        uri = %request.uri().path(),
                        req = %request.method(),
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {})
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    let s = response.status();
                    match latency.as_micros() {
                        0..=999 => debug!("[{}] Served in {}μs", s, latency.as_micros()),
                        1000..=999999 => debug!("[{}] Served in {}ms", s, latency.as_millis()),
                        _ => debug!("[{}] Served in {}s", s, latency.as_secs()),
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
