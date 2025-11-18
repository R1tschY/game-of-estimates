use crate::web::embed::{get_asset_url, AssetCatalog, EmbedTemplateContext};
use crate::web::handlebars::{Template, TemplateRenderer, TemplateResult};
use crate::web::headers::Language;
use crate::web::i18n::LanguageNegotiator;
use crate::web::i18n::Translator;
use crate::web::metrics::handler::serve_metrics;
use crate::web::metrics::prometheus::RequestMetrics;
use crate::web::metrics::service::RequestMetricsLayer;
use crate::ListenAddr;
use ::handlebars::Handlebars;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, Request, State, WebSocketUpgrade};
use axum::response::{ErrorResponse, Redirect, Response};
use axum::routing::{any, post};
use axum::{routing::get, Extension, Form, Router};
use game_of_estimates::game_server::{GameServerAddr, GameServerMessage};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use http::StatusCode;
use log::error;
use prometheus_client::registry::Registry;
use rust_embed::Embed;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

pub mod embed;
pub mod handlebars;
pub mod headers;
pub mod i18n;
mod metrics;

#[derive(Embed)]
#[folder = "frontend/dist/"]
pub struct MyAssetCatalog;

pub struct AppState {
    game_server: GameServerAddr,
    hbs: TemplateRenderer,
}

async fn lobby(lang: Language, State(state): State<Arc<AppState>>) -> TemplateResult {
    state.hbs.render(Template::html(
        "lobby.html",
        json!({
            "lang": lang,
        }),
    ))
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

async fn room(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    lang: Language,
) -> TemplateResult {
    state.hbs.render(Template::html(
        "room.html",
        json!({
            "lang": lang,
            "roomId": id,
            "js": get_asset_url::<MyAssetCatalog>("assets/room.js"),
            "css": get_asset_url::<MyAssetCatalog>("assets/style.css"),
        }),
    ))
}

async fn websocket(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    let game_server = state.game_server.clone();
    ws.on_upgrade(|socket: WebSocket| async {
        Player::new(RemoteConnection::new(socket), game_server)
            .run()
            .await
    })
}

async fn assets(req: Request) -> Response {
    AssetCatalog::<MyAssetCatalog>::new().serve(&req)
}

pub async fn main(game_server: GameServerAddr, listen_addr: ListenAddr) {
    // i18n
    let mut translations = HashMap::new();
    translations.insert(
        "de",
        serde_json::de::from_str(include_str!("../../frontend/src/i18n/de.json"))
            .expect("german translation should be valid"),
    );
    translations.insert(
        "en",
        serde_json::de::from_str(include_str!("../../frontend/src/i18n/en.json"))
            .expect("english translation should be valid"),
    );
    let lang = LanguageNegotiator::new(&translations);
    let translator = Translator::new(translations);

    let mut hbs = Handlebars::new();
    #[cfg(debug_assertions)]
    hbs.set_dev_mode(true);
    let templates_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates");
    hbs.register_templates_directory(templates_dir, Default::default())
        .expect("templates should be valid");
    hbs.register_helper("text", Box::new(translator.clone()));
    hbs.register_helper(
        "asset",
        Box::new(EmbedTemplateContext::<MyAssetCatalog>::new()),
    );

    let mut registry = Registry::default();
    let req_metrics = RequestMetrics::new(&mut registry);

    let app = Router::new()
        .route("/", get(lobby))
        .route("/room", post(create_room))
        .route("/room/{id}", get(room))
        .route("/ws", any(websocket))
        .route("/metrics", get(serve_metrics(Arc::new(registry))))
        .fallback(get(assets))
        .with_state(Arc::new(AppState {
            game_server,
            hbs: TemplateRenderer::new(hbs),
        }))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(lang))
                .layer(CompressionLayer::new())
                .layer(RequestMetricsLayer::new(req_metrics)),
        );

    let listener = tokio::net::TcpListener::bind(listen_addr.0)
        .await
        .expect("should bind to listen address");
    axum::serve(listener, app).await.unwrap();
}
