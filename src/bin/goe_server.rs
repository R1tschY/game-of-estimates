use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use handlebars::Handlebars;
use log::info;
use serde::{Deserialize, Serialize};
use warp::http::Uri;
use warp::Filter;

use game_of_estimates::game_server::{GameServer, GameServerMessage, ReturnEnvelope};
use uactor::blocking::{Actor, Addr};

trait ErrorContextExt<T> {
    fn in_context(self, context: &str) -> Result<T, String>;
}

impl<T, E: std::fmt::Debug> ErrorContextExt<T> for Result<T, E> {
    fn in_context(self, context: &str) -> Result<T, String> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(format!("{}: {:?}", context, err)),
        }
    }
}

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, templates: Arc<TemplateSystem<'_>>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = templates
        .render(template.name, &template.value)
        .unwrap_or_else(|err| format!("{:?}", err));
    warp::reply::html(render)
}

struct TemplateSystem<'a> {
    template_dir: PathBuf,
    hbs: Handlebars<'a>,
}

impl<'a> TemplateSystem<'a> {
    pub fn from_env() -> Result<Self, Box<dyn Debug>> {
        let template_dir = std::env::var("GOE_TEMPLATE_DIR").map_err(|err| {
            Box::new(format!(
                "GOE_TEMPLATE_DIR env var is missing or invalid: {:?}",
                err
            )) as Box<dyn Debug>
        })?;

        let mut hbs = Handlebars::new();
        hbs.set_dev_mode(true);

        Ok(Self {
            template_dir: template_dir.into(),
            hbs,
        })
    }

    pub fn register_template(&mut self, name: &str) -> Result<(), Box<dyn Debug>> {
        self.hbs
            .register_template_file(name, self.template_dir.join(format!("{}.hbs", name)))
            .map_err(|err| {
                Box::new(format!("invalid template {}: {:?}", name, err)) as Box<dyn Debug>
            })
    }

    pub fn render<T: Serialize>(&self, name: &str, data: &T) -> Result<String, Box<dyn Debug>> {
        self.hbs
            .render(name, data)
            .map_err(|err| Box::new(err.to_string()) as Box<dyn Debug>)
    }

    pub fn template_dir(&self) -> &Path {
        &self.template_dir
    }
}

#[derive(Serialize)]
struct Deck {
    value: &'static str,
    display: &'static str,
}

#[derive(Serialize)]
struct LobbyData {
    decks: Vec<Deck>,
}

#[derive(Serialize)]
struct RoomData {
    room_id: String,
}

#[derive(Deserialize)]
struct CreateRoomData {
    deck: String,
}

mod wrapx {
    use warp::Filter;

    pub fn with_data<T: Clone + Send>(
        value: T,
    ) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || value.clone())
    }
}

async fn create_room(
    data: CreateRoomData,
    game_server: Addr<GameServerMessage>,
) -> impl warp::Reply {
    let (ret, revc) = ReturnEnvelope::channel();

    if game_server
        .send(GameServerMessage::ExternalCreate {
            deck: data.deck,
            ret,
        })
        .await
        .is_err()
    {
        return warp::redirect::see_other(Uri::from_static("/service_unavailable"));
    }

    match revc.await {
        Ok(Ok(room_id)) => {
            warp::redirect::see_other(Uri::from_maybe_shared(format!("/room/{}", room_id)).unwrap())
        }
        Ok(Err(_err)) => warp::redirect::see_other(Uri::from_static("/error")),
        Err(_err) => warp::redirect::see_other(Uri::from_static("/service_unavailable")),
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::try_init().in_context("Failed to init logger")?;

    let mut template_sys =
        TemplateSystem::from_env().in_context("Failed to init template system")?;
    template_sys
        .register_template("base")
        .in_context("Failed to register template")?;

    let template_sys = Arc::new(template_sys);
    let tsys = template_sys.clone();
    let handlebars = move |with_template| render(with_template, template_sys.clone());

    let data: &'static LobbyData = Box::leak(Box::new(LobbyData {
        decks: vec![
            Deck {
                value: "mod-fibonacci",
                display: "Modified Fibonacci (0, ½, 1, 2, 3, 5, 8, 13, 20, 40, 100)",
            },
            Deck {
                value: "fibonacci",
                display: "Fibonacci (0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89)",
            },
            Deck {
                value: "t-shirt-sizes",
                display: "T-shirt Sizes (XS, S, M, L, XL, XXL)",
            },
            Deck {
                value: "power-of-2",
                display: "Powers of 2 (0, 1, 2, 4, 8, 16, 32, 64)",
            },
        ],
    }));
    let game_server = GameServer::default().start();

    // GET /
    let index = warp::path::end()
        .map(move || WithTemplate {
            name: "base",
            value: data,
        })
        .map(handlebars);

    // GET /room/{}
    let tsys2 = tsys.clone();
    let room = warp::path("room")
        .and(warp::path::param())
        .and(warp::path::end())
        .map(move |room_id: String| WithTemplate {
            name: "room",
            value: RoomData { room_id },
        })
        .map(move |with_template| render(with_template, tsys2.clone()));

    // POST /room/create
    let create_room = warp::path("room")
        .and(warp::path("create"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and(wrapx::with_data(game_server.clone()))
        .then(create_room);

    // *.js/*.css
    let bundle_js = warp::path("bundle.js")
        .and(warp::path::end())
        .and(warp::fs::file(tsys.template_dir().join("bundle.js")));
    let bundle_css = warp::path("bundle.css")
        .and(warp::path::end())
        .and(warp::fs::file(tsys.template_dir().join("bundle.css")));

    let route = warp::get()
        .and(index.or(bundle_js).or(bundle_css).or(room))
        .or(create_room);

    info!("HTTP server: http://{}:{}", "localhost", 3030);
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
