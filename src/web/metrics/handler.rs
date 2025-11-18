use axum::body::Body;
use axum::handler::Handler;
use axum::response::IntoResponse;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::Arc;

pub fn serve_metrics<S>(registry: Arc<Registry>) -> impl Handler<((),), S> {
    || async move {
        let mut buffer = String::new();
        match encode(&mut buffer, &registry) {
            Ok(()) => {
                let mut resp = Body::from(buffer).into_response();
                resp.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static(
                        "application/openmetrics-text; version=1.0.0; charset=utf-8",
                    ),
                );
                resp
            }
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
