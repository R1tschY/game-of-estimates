use crate::web::metrics::service::{RequestMetricEmitter, RequestMetricsEmitterFactory};
use axum::extract::{MatchedPath, Request};
use http::Response;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::fmt::Error;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct RequestMetrics(Arc<InnerRequestMetrics>);

struct InnerRequestMetrics {
    http_requests_total: Family<RequestLabels, Counter>,
    http_requests_duration_seconds: Family<RequestLabels, Histogram>,
}

impl RequestMetrics {
    pub fn new(registry: &mut Registry) -> Self {
        let http_requests_total = Family::<RequestLabels, Counter>::default();
        registry.register(
            "http_requests_total",
            "Number of HTTP requests received",
            http_requests_total.clone(),
        );

        let http_requests_duration_seconds =
            Family::<RequestLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(
                    [0.05f64, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 10.0]
                        .iter()
                        .copied(),
                )
            });
        registry.register(
            "http_requests_duration_seconds",
            "Number of HTTP requests received",
            http_requests_duration_seconds.clone(),
        );

        Self(Arc::new(InnerRequestMetrics {
            http_requests_total,
            http_requests_duration_seconds,
        }))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
enum Method {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Other,
}

impl From<&'_ http::Method> for Method {
    fn from(value: &'_ http::Method) -> Self {
        match *value {
            http::Method::OPTIONS => Method::Options,
            http::Method::GET => Method::Get,
            http::Method::POST => Method::Post,
            http::Method::PUT => Method::Put,
            http::Method::DELETE => Method::Delete,
            http::Method::HEAD => Method::Head,
            http::Method::TRACE => Method::Trace,
            http::Method::CONNECT => Method::Connect,
            http::Method::PATCH => Method::Patch,
            _ => Method::Other,
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq, EncodeLabelSet, Debug)]
struct RequestLabels {
    path: Path,
    method: Method,
    status: u16,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
enum Path {
    Route(Route),
    Fallback(String),
}

#[derive(Clone, Debug)]
struct Route(MatchedPath);

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Eq for Route {}

impl Hash for Route {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.0.as_str(), state)
    }
}

impl EncodeLabelValue for Path {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), Error> {
        match self {
            Path::Route(route) => LabelValueEncoder::write_str(encoder, route.0.as_str())?,
            Path::Fallback(path) => LabelValueEncoder::write_str(encoder, path.as_str())?,
        }
        Ok(())
    }
}

impl RequestMetricsEmitterFactory for RequestMetrics {
    type RequestBody = axum::body::Body;
    type ResponseBody = axum::body::Body;
    type Emitter = GoeRequestMetricsEmitter;

    fn new_emitter(&self, request: &Request<Self::RequestBody>) -> Self::Emitter {
        let path = if let Some(path) = request.extensions().get::<MatchedPath>() {
            Path::Route(Route(path.clone()))
        } else {
            Path::Fallback(request.uri().path().to_string())
        };

        GoeRequestMetricsEmitter {
            start_time: Instant::now(),
            path,
            method: request.method().into(),
            status: 0,
            metrics: self.0.clone(),
        }
    }
}

pub struct GoeRequestMetricsEmitter {
    start_time: Instant,
    path: Path,
    method: Method,
    status: u16,
    metrics: Arc<InnerRequestMetrics>,
}

impl RequestMetricEmitter for GoeRequestMetricsEmitter {
    type ResponseBody = axum::body::Body;

    fn handle_response(&mut self, response: &Response<Self::ResponseBody>) {
        self.status = response.status().as_u16();
    }

    fn emit(self) {
        let labels = RequestLabels {
            path: self.path,
            method: self.method,
            status: self.status,
        };

        self.metrics
            .http_requests_duration_seconds
            .get_or_create(&labels)
            .observe(self.start_time.elapsed().as_secs_f64());

        self.metrics
            .http_requests_total
            .get_or_create(&labels)
            .inc();
    }
}
