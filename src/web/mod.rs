use crate::web::compress::{Compression, DefaultPredicate};
use crate::web::embed::{get_asset_url, AssetCatalog};
use crate::web::handlebars::Template;
use crate::web::headers::Language;
use crate::web::i18n::AcceptLanguageHelper;
use crate::web::i18n::I18nHelper;
use crate::web::prometheus::Prometheus;
use crate::web::see_other::SeeOther;
use ::handlebars::Handlebars;
use async_compression::Level;
use game_of_estimates::game_server::{GameServerAddr, GameServerMessage};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use log::error;
use prometheus_client::registry::Registry;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Figment;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{routes, Build, FromForm, Rocket, State};
use rocket_ws::WebSocket;
use rust_embed::Embed;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

pub mod compress;
pub mod embed;
pub mod handlebars;
pub mod headers;
pub mod i18n;
pub mod prometheus;
pub mod see_other;

#[derive(Embed)]
#[folder = "frontend/build/"]
pub struct MyAssetCatalog;

#[rocket::get("/")]
fn lobby(lang: Language) -> Template {
    Template::html(
        "lobby.html",
        json!({
            "lang": lang,
            "js": get_asset_url::<MyAssetCatalog>("assets/createRoom.js"),
            "css": get_asset_url::<MyAssetCatalog>("assets/style.css"),
        }),
    )
}

#[derive(FromForm)]
struct CreateRoomFormData<'r> {
    deck: &'r str,
    custom_deck: Option<&'r str>,
}

#[rocket::post("/room", data = "<data>")]
async fn create_room(
    data: Form<CreateRoomFormData<'_>>,
    game_server_addr: &State<GameServerAddr>,
) -> Result<SeeOther, Status> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let deck: String = if data.deck == "custom" {
        match data.custom_deck {
            Some(custom_deck) => format!("custom:{custom_deck}"),
            None => {
                error!("missing custom deck form field");
                return Err(Status::UnprocessableEntity);
            }
        }
    } else {
        data.deck.to_string()
    };
    let res = game_server_addr
        .inner()
        .send(GameServerMessage::Create { deck, reply: tx })
        .await;
    if res.is_err() {
        error!("Failed to create room: game service is offline");
        return Err(Status::ServiceUnavailable);
    }

    match rx.await {
        Ok(room_id) => Ok(SeeOther::new(format!("/room/{}", room_id))),
        Err(_) => {
            error!("Failed to create room: game service dropped message");
            Err(Status::ServiceUnavailable)
        }
    }
}

#[rocket::get("/room/<id>")]
fn room(lang: Language, id: &str) -> Template {
    Template::html(
        "room.html",
        json!({
            "lang": lang,
            "roomId": id,
            "js": get_asset_url::<MyAssetCatalog>("assets/room.js"),
            "css": get_asset_url::<MyAssetCatalog>("assets/style.css"),
        }),
    )
}

#[rocket::get("/ws")]
async fn websocket(
    ws: WebSocket,
    game_server: &State<GameServerAddr>,
) -> rocket_ws::Channel<'static> {
    let game_server = game_server.inner().clone();
    ws.channel(move |ws| {
        Box::pin(async move {
            Player::new(RemoteConnection::new(ws), game_server)
                .run()
                .await;
            Ok(())
        })
    })
}

pub async fn rocket(game_server: GameServerAddr) -> Rocket<Build> {
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
    let trans_helper = AcceptLanguageHelper::new(&translations);
    let i18n_helper = I18nHelper::new(translations);

    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file("game_of_estimates.toml").nested())
        .merge(Env::prefixed("GOE_").global());

    rocket::custom(figment)
        .manage(Template::state({
            let mut hbs = Handlebars::new();
            #[cfg(debug_assertions)]
            hbs.set_dev_mode(true);
            let templates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates");
            hbs.register_templates_directory(templates_dir, Default::default())
                .expect("templates should be valid");
            hbs.register_helper("text", Box::new(i18n_helper.clone()));
            hbs
        }))
        .attach(Prometheus::new(Registry::default()))
        .attach(Compression::new(
            Level::Default,
            DefaultPredicate::default(),
        ))
        .manage(trans_helper)
        .manage(game_server)
        .mount("/", routes![lobby, create_room, room, websocket])
        .mount("/", AssetCatalog::<MyAssetCatalog>::new())
}
