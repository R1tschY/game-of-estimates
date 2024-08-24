use crate::web::embed::AssetCatalog;
use crate::web::handlebars::Template;
use crate::web::headers::Language;
use crate::web::i18n::AcceptLanguageHelper;
use crate::web::i18n::I18nHelper;
use crate::web::see_other::SeeOther;
use ::handlebars::Handlebars;
use game_of_estimates::game_server::{GameServerAddr, GameServerMessage};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use log::error;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Figment;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::{routes, Build, FromForm, Rocket, State};
use rocket_ws::WebSocket;
use rust_embed::Embed;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

pub mod embed;
pub mod handlebars;
pub mod headers;
pub mod i18n;
pub mod see_other;

#[derive(Embed)]
#[folder = "frontend/dist/"]
pub struct MyAssetCatalog;

#[rocket::get("/")]
fn lobby(lang: Language) -> Template {
    Template::html(
        "lobby.html",
        json!({
            "lang": lang,
            "js": "/assets/createRoom.js",
            "css": "/assets/style.css",
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
    if let Err(_) = res {
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

#[rocket::get("/room")]
fn room(lang: Language) -> Template {
    Template::html(
        "room.html",
        json!({
            "lang": lang,
            "js": "/assets/room.js",
            "css": "/assets/style.css",
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
            Ok(Player::new(RemoteConnection::new(ws), game_server)
                .run()
                .await)
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
            let templates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates");
            hbs.register_templates_directory(templates_dir, Default::default())
                .expect("templates should be valid");
            hbs.register_helper("text", Box::new(i18n_helper.clone()));
            hbs
        }))
        .manage(trans_helper)
        .manage(game_server)
        .mount("/", routes![lobby, create_room, room, websocket])
        .mount("/", AssetCatalog::<MyAssetCatalog>::new())
}
