use crate::web::metrics::handler::serve_metrics;
use crate::web::metrics::prometheus::RequestMetrics;
use crate::web::metrics::service::RequestMetricsLayer;
use crate::ListenAddr;
use axum::body::Body;
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{ErrorResponse, IntoResponse, Redirect, Response};
use axum::routing::{any, post};
use axum::{routing::get, Form, Router};
use game_of_estimates::game_server::{GameServerAddr, GameServerMessage};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use http::{HeaderValue, StatusCode};
use log::error;
use prometheus_client::registry::Registry;
use rust_embed::Embed;
use serde::Deserialize;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_serve_assets::embed::EmbedCatalog;
use tower_serve_assets::ServeAssets;

pub mod headers;
pub mod i18n;
mod metrics;

#[derive(Embed)]
#[folder = "frontend/build/"]
pub struct MyAssetCatalog;

pub struct AppState {
    game_server: GameServerAddr,
}

#[derive(Deserialize)]
struct CreateRoomFormData {
    deck: String,
    custom_deck: Option<String>,
}

async fn create_room(
    State(state): State<Arc<AppState>>,
    Form(data): Form<CreateRoomFormData>,
) -> Result<Redirect, ErrorResponse> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let deck: String = if data.deck == "custom" {
        match data.custom_deck {
            Some(custom_deck) => format!("custom:{custom_deck}"),
            None => {
                error!("missing custom deck form field");
                return Err(StatusCode::UNPROCESSABLE_ENTITY.into());
            }
        }
    } else {
        data.deck.to_string()
    };
    let res = state
        .game_server
        .send(GameServerMessage::Create { deck, reply: tx })
        .await;
    if res.is_err() {
        error!("Failed to create room: game service is offline");
        return Err(StatusCode::SERVICE_UNAVAILABLE.into());
    }

    match rx.await {
        Ok(Some(room_id)) => Ok(Redirect::to(&format!("/room/{room_id}"))),
        Ok(None) => Err(StatusCode::SERVICE_UNAVAILABLE.into()),
        Err(_) => {
            error!("Failed to create room: game service dropped message");
            Err(StatusCode::SERVICE_UNAVAILABLE.into())
        }
    }
}

async fn room() -> Response {
    // TODO: etag caching
    if let Some(asset) = MyAssetCatalog::get("200.html") {
        let mut response = Body::from(asset.data).into_response();
        response.headers_mut().append(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        response
    } else {
        (StatusCode::NOT_FOUND, "Asset not found").into_response()
    }
}

async fn websocket(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    let game_server = state.game_server.clone();
    ws.on_upgrade(|socket: WebSocket| async {
        Player::new(RemoteConnection::new(socket), game_server)
            .run()
            .await
    })
}

pub async fn main(game_server: GameServerAddr, listen_addr: ListenAddr) {
    // i18n
    let mut registry = Registry::default();
    let req_metrics = RequestMetrics::new(&mut registry);

    let app = Router::new()
        .route("/room", post(create_room))
        .route("/room/{id}", get(room))
        .route("/ws", any(websocket))
        .route("/metrics", get(serve_metrics(Arc::new(registry))))
        .fallback_service(ServeAssets::builder(EmbedCatalog::<MyAssetCatalog>::default()).build())
        .with_state(Arc::new(AppState { game_server }))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(RequestMetricsLayer::new(req_metrics)),
        );

    let listener = tokio::net::TcpListener::bind(listen_addr.0)
        .await
        .expect("should bind to listen address");
    axum::serve(listener, app).await.unwrap();
}
