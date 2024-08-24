use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{ContentType, Method, Status};
use rocket::route::{Handler, Outcome};
use rocket::{Build, Data, Request, Response, Rocket, Route};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;

pub struct Prometheus {
    registry: Arc<Registry>,
    http_requests_total: Family<Labels, Counter>,
    http_requests_duration_seconds: Family<Labels, Histogram>,
}

#[derive(Clone)]
struct PrometheusHandler {
    registry: Arc<Registry>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct Labels {
    endpoint: String,
    method: &'static str,
    status: u16,
}

#[derive(Copy, Clone)]
struct TimerStart(Option<Instant>);

impl Prometheus {
    pub fn new(mut registry: Registry) -> Self {
        let http_requests_total = Family::<Labels, Counter>::default();
        registry.register(
            "rocket_http_requests_total",
            "Number of HTTP requests received",
            http_requests_total.clone(),
        );

        let http_requests_duration_seconds =
            Family::<Labels, Histogram>::new_with_constructor(|| {
                Histogram::new(
                    [0.05f64, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 10.0]
                        .iter()
                        .copied(),
                )
            });
        registry.register(
            "rocket_http_requests_duration_seconds",
            "Number of HTTP requests received",
            http_requests_duration_seconds.clone(),
        );

        Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_requests_duration_seconds,
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn http_requests_total(&self) -> &Family<Labels, Counter> {
        &self.http_requests_total
    }

    pub fn http_requests_duration_seconds(&self) -> &Family<Labels, Histogram> {
        &self.http_requests_duration_seconds
    }
}

#[rocket::async_trait]
impl Fairing for Prometheus {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus metric collector",
            kind: Kind::Request | Kind::Response | Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        let mut route = Route::new(
            Method::Get,
            "/metrics",
            PrometheusHandler {
                registry: self.registry.clone(),
            },
        );
        route.name = Some("Prometheus metrics".into());
        Ok(rocket.mount("/", vec![route]))
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        req.local_cache(|| TimerStart(Some(Instant::now())));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        if let Some(route) = req.route() {
            let endpoint = route.uri.as_str().to_string();
            let method = req.method().as_str();
            let status = res.status().code;

            let labels = Labels {
                method,
                endpoint,
                status,
            };

            self.http_requests_total.get_or_create(&labels).inc();

            if let Some(start_time) = req.local_cache(|| TimerStart(None)).0 {
                self.http_requests_duration_seconds
                    .get_or_create(&labels)
                    .observe(start_time.elapsed().as_secs_f64());
            }
        }
    }
}

#[rocket::async_trait]
impl Handler for PrometheusHandler {
    async fn handle<'r>(&self, _: &'r Request<'_>, _: Data<'r>) -> Outcome<'r> {
        let mut buffer = String::new();
        match encode(&mut buffer, &self.registry) {
            Ok(_) => Outcome::Success(
                Response::build()
                    .raw_header(
                        "Content-Type",
                        "application/openmetrics-text; version=1.0.0; charset=utf-8",
                    )
                    .sized_body(buffer.len(), Cursor::new(buffer))
                    .finalize(),
            ),
            Err(_) => Outcome::Success(
                Response::build()
                    .status(Status::InternalServerError)
                    .finalize(),
            ),
        }
    }
}
