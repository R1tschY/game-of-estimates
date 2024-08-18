use game_of_estimates::adapters::sqlx::SqlxModule;
use game_of_estimates::assets;
use game_of_estimates::assets::AssetCatalog;
use game_of_estimates::game_server::{GameServer, GameServerAddr, GameServerMessage};
use game_of_estimates::player::Player;
use game_of_estimates::ports::{DatabaseMigratorRef, DatabaseUrl, RoomRepositoryRef};
use game_of_estimates::remote::RemoteConnection;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, PathAndJson, RenderContext,
    RenderError, RenderErrorReason, ScopedJson,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::num::ParseFloatError;
use std::path::Path;
#[cfg(feature = "tls")]
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fmt};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot::error::RecvError;
use uactor::blocking::Actor;
use warp::http::{Response, StatusCode};
use warp::ws::WebSocket;
use warp::Filter;

trait ErrorContextExt<T> {
    fn in_context(self, context: &str) -> Result<T, String>;
}

impl<T, E: fmt::Display> ErrorContextExt<T> for Result<T, E> {
    fn in_context(self, context: &str) -> Result<T, String> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(format!("{}: {}", context, err)),
        }
    }
}

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars<'_>>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}

#[derive(Deserialize)]
struct Translation {
    language: String,
    strings: HashMap<String, String>,
}

#[derive(Clone)]
struct AcceptLanguageHelper {
    translations: Arc<Vec<&'static str>>,
}

impl AcceptLanguageHelper {
    fn new(translations: &HashMap<&'static str, Translation>) -> Self {
        Self {
            translations: Arc::new(translations.iter().map(|x| *x.0).collect()),
        }
    }

    fn get_language(&self, languages: Option<AcceptLanguage>) -> &'static str {
        languages
            .and_then(|langs| {
                langs.languages.iter().find_map(|lang| {
                    self.translations
                        .iter()
                        .copied()
                        .find(|&trans| trans == lang)
                })
            })
            .unwrap_or("en")
    }
}

struct I18nHelper {
    translations: HashMap<&'static str, Translation>,
}

impl HelperDef for I18nHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let arg = match helper.param(0) {
            None => Err(RenderError::from(RenderErrorReason::ParamNotFoundForIndex(
                "text", 0,
            ))),
            Some(param) => match param.value() {
                Value::String(param) => Ok(param),
                _ => Err(RenderError::from(RenderErrorReason::Other(
                    "Helper text param at index 0 has invalid param type, string expected"
                        .to_string(),
                ))),
            },
        }?;

        let lang = match ctx.data() {
            Value::Object(ctx) => match ctx.get("lang") {
                Some(Value::String(lang)) => Ok(lang),
                _ => Err(RenderError::from(RenderErrorReason::Other(
                    "Helper text ctx param lang required".to_string(),
                ))),
            },
            _ => Err(RenderError::from(RenderErrorReason::Other(
                "Helper text requires context to be an object".to_string(),
            ))),
        }?;

        let translation = self.translations.get(lang.as_str()).unwrap();
        let text = translation.strings.get(arg).ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(format!(
                "Helper text argument '{}' is not a known string for {} translation",
                arg, "de"
            )))
        })?;

        Ok(Value::String(text.to_owned()).into())
    }
}

struct AcceptLanguage {
    languages: Vec<String>,
}

impl FromStr for AcceptLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut langs = s
            .split(",")
            .map(|elem| {
                let x = elem.trim_matches(|c: char| matches!(c, ' ' | '\x09'));
                Ok(match x.split_once(';') {
                    Some((lang, weight)) => (
                        lang,
                        match f32::from_str(
                            weight
                                .trim_matches(|c: char| matches!(c, ' ' | '\x09'))
                                .strip_prefix("q=")
                                .ok_or(())?,
                        ) {
                            Ok(w) => w,
                            Err(_) => return Err(()),
                        },
                    ),
                    None => (x, 1.0f32),
                })
            })
            .collect::<Result<Vec<(&str, f32)>, Self::Err>>()?;

        langs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok(AcceptLanguage {
            languages: langs.iter().map(|x| x.0.to_string()).collect(),
        })
    }
}

#[derive(Default)]
struct MainModule;

#[chassis::module]
impl MainModule {
    pub fn provide_database_url() -> DatabaseUrl {
        DatabaseUrl(std::env::var("DATABASE_URL").expect("DATABASE_URL should be set"))
    }

    #[chassis(singleton)]
    pub fn provide_game_server(room_repo: RoomRepositoryRef) -> GameServerAddr {
        GameServer::new(room_repo).start()
    }

    #[chassis(singleton)]
    pub fn provide_cert_paths() -> TlsCert {
        #[cfg(feature = "tls")]
        if let Some(cert_path) = env::var_os("GOE_PEM_PATH") {
            if let Some(key_path) = env::var_os("GOE_KEY_PATH") {
                TlsCert::Pem {
                    cert_path: cert_path.into(),
                    key_path: key_path.into(),
                }
            } else {
                TlsCert::Unencrypted
            }
        } else {
            TlsCert::Unencrypted
        }
        #[cfg(not(feature = "tls"))]
        TlsCert::Unencrypted
    }

    #[chassis(singleton)]
    pub fn provide_listen_addr() -> ListenAddr {
        ListenAddr(env::var("GOE_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:5500".to_string()))
    }

    pub fn provide_main(addr: ListenAddr, tls_cert: TlsCert, game_server: GameServerAddr) -> Main {
        Main {
            addr,
            tls_cert,
            game_server,
        }
    }
}

#[chassis::injector(modules = [MainModule, SqlxModule])]
pub trait Integrator {
    fn provide_main(&self) -> Main;
    fn provide_db_migrator(&self) -> DatabaseMigratorRef;
}

pub fn data<T: Clone + Send>(value: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || value.clone())
}

#[derive(Clone)]
struct ListenAddr(String);

#[derive(Clone)]
enum TlsCert {
    Unencrypted,
    #[cfg(feature = "tls")]
    Pem {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

pub struct Main {
    addr: ListenAddr,
    tls_cert: TlsCert,
    game_server: GameServerAddr,
}

impl Main {
    pub async fn run(&self) -> Result<(), String> {
        let addr: SocketAddr = self.addr.0.parse().expect("Invalid listen address");
        let assets = Box::leak(Box::new(AssetCatalog::new()));

        let ws = warp::path("ws")
            .and(warp::path::end())
            .and(warp::ws())
            .and(data(self.game_server.clone()))
            .map(|ws: warp::ws::Ws, game_server: GameServerAddr| {
                ws.on_upgrade(move |websocket| Main::run_player(websocket, game_server))
            });

        // i18n
        let mut translations = HashMap::new();
        translations.insert(
            "de",
            serde_json::de::from_str(include_str!("../frontend/src/i18n/de.json"))
                .expect("german translation should be valid"),
        );
        translations.insert(
            "en",
            serde_json::de::from_str(include_str!("../frontend/src/i18n/en.json"))
                .expect("english translation should be valid"),
        );
        let trans_helper = AcceptLanguageHelper::new(&translations);
        let i18n_helper = I18nHelper { translations };

        // handlebars
        let mut hb = Handlebars::new();
        hb.set_dev_mode(true);
        let templates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates");
        hb.register_templates_directory(templates_dir, Default::default())
            .expect("templates should be valid");
        hb.register_helper("text", Box::new(i18n_helper));

        let hb = Arc::new(hb);
        let handlebars = move |with_template| render(with_template, hb.clone());

        // Lobby
        let trans_helper_ = trans_helper.clone();
        let lobby = warp::get()
            .and(warp::path::end())
            .and(warp::header::optional("Accept-Language"))
            .map(move |accept_language: Option<AcceptLanguage>| {
                let lang = trans_helper_.get_language(accept_language);
                WithTemplate {
                    name: "lobby.html",
                    value: json!({
                        "lang": lang,
                        "js": "/assets/createRoom.js",
                        "css": "/assets/style.css",
                    }),
                }
            })
            .map(handlebars.clone());

        // Create Room
        #[derive(Deserialize)]
        struct CreateRoomFormData {
            deck: String,
            custom_deck: Option<String>,
        }

        let create_room = warp::post()
            .and(warp::path("room"))
            .and(warp::path("create"))
            .and(warp::path::end())
            .and(warp::body::content_length_limit(1024 * 32))
            .and(warp::body::form())
            .and(data(self.game_server.clone()))
            .then(
                move |data: CreateRoomFormData, game_server: GameServerAddr| async move {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let deck = if data.deck == "custom" {
                        match data.custom_deck {
                            Some(custom_deck) => format!("custom:{custom_deck}"),
                            None => {
                                return Response::builder()
                                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                                    .body("custom_deck parameter should be set")
                            }
                        }
                    } else {
                        data.deck
                    };
                    let res = game_server
                        .send(GameServerMessage::Create { deck, reply: tx })
                        .await;
                    if let Err(err) = res {
                        error!("Failed to create room: game service is offline");
                        return Response::builder()
                            .status(StatusCode::SERVICE_UNAVAILABLE)
                            .body("");
                    }

                    match rx.await {
                        Ok(room_id) => Response::builder()
                            .status(StatusCode::SEE_OTHER)
                            .header("Location", format!("/room/{}", room_id))
                            .body(""),
                        Err(err) => {
                            error!("Failed to create room: game service dropped message");
                            Response::builder()
                                .status(StatusCode::SERVICE_UNAVAILABLE)
                                .body("")
                        }
                    }
                },
            );

        // Room
        let room = warp::get()
            .and(warp::path("room"))
            .and(warp::path::param())
            .and(warp::path::end())
            .and(warp::header::optional("Accept-Language"))
            .map(move |id: String, accept_language: Option<AcceptLanguage>| {
                let lang = trans_helper.get_language(accept_language);
                WithTemplate {
                    name: "room.html",
                    value: json!({
                        "lang": lang,
                        "js": "/assets/room.js",
                        "css": "/assets/style.css",
                        "roomId": id
                    }),
                }
            })
            .map(handlebars);

        info!("Listening on {} ...", &self.addr.0);
        let routes = lobby
            .or(create_room)
            .or(room)
            .or(ws)
            .or(assets::assets(assets));
        match self.tls_cert.clone() {
            TlsCert::Unencrypted => warp::serve(routes).run(addr).await,
            #[cfg(feature = "tls")]
            TlsCert::Pem {
                cert_path,
                key_path,
            } => {
                warp::serve(routes)
                    .tls()
                    .cert_path(cert_path)
                    .key_path(key_path)
                    .run(addr)
                    .await;
            }
        }

        Ok(())
    }

    async fn run_player(ws: WebSocket, game_server: GameServerAddr) {
        Player::new(RemoteConnection::new(ws), game_server)
            .run()
            .await
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    dotenvy::dotenv().in_context("Unable to load .env files")?;
    env_logger::try_init().in_context("Unable to initialize logger")?;

    let mut sigterm =
        signal(SignalKind::terminate()).in_context("Unable to register SIGTERM handler")?;
    let mut sigint =
        signal(SignalKind::interrupt()).in_context("Unable to register SIGINT handler")?;

    let integrator = <dyn Integrator>::new().in_context("Unable to build injector")?;

    integrator
        .provide_db_migrator()
        .migrate()
        .await
        .in_context("Unable to migrate database")?;

    let main = integrator.provide_main();

    tokio::select! {
        ret = main.run() => { ret },

        _ = sigterm.recv() => {
            info!("Received SIGTERM. Shutting down ...");
            Ok(())
        },
        _ = sigint.recv() => {
            info!("Received SIGINT. Shutting down ...");
            Ok(())
        },
    }
}
