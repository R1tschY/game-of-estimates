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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

impl prometheus_client::encoding::EncodeLabelValue for Method {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), Error> {
        use std::fmt::Write;
        match self {
            Method::Options => encoder.write_str("OPTIONS")?,
            Method::Get => encoder.write_str("GET")?,
            Method::Post => encoder.write_str("POST")?,
            Method::Put => encoder.write_str("PUT")?,
            Method::Delete => encoder.write_str("DELETE")?,
            Method::Head => encoder.write_str("HEAD")?,
            Method::Trace => encoder.write_str("TRACE")?,
            Method::Connect => encoder.write_str("CONNECT")?,
            Method::Patch => encoder.write_str("PATCH")?,
            Method::Other => encoder.write_str("other")?,
        }
        Ok(())
    }
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
    route: Option<Route>,
    method: Method,
    status: u16,
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

impl EncodeLabelValue for Route {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), Error> {
        LabelValueEncoder::write_str(encoder, self.0.as_str())
    }
}

impl RequestMetricsEmitterFactory for RequestMetrics {
    type RequestBody = axum::body::Body;
    type ResponseBody = axum::body::Body;
    type Emitter = GoeRequestMetricsEmitter;

    fn new_emitter(&self, request: &Request<Self::RequestBody>) -> Self::Emitter {
        GoeRequestMetricsEmitter {
            start_time: Instant::now(),
            route: request
                .extensions()
                .get::<MatchedPath>()
                .map(|path| Route(path.clone())),
            method: request.method().into(),
            status: 0,
            metrics: self.0.clone(),
        }
    }
}

pub struct GoeRequestMetricsEmitter {
    start_time: Instant,
    route: Option<Route>,
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
            route: self.route,
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
