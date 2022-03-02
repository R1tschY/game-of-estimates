use std::fmt::{format, Debug};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use warp::Filter;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Debug>> {
    let mut template_sys = TemplateSystem::from_env()?;
    template_sys.register_template("base")?;

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

    //GET /
    let index = warp::path::end()
        .map(move || WithTemplate {
            name: "base",
            value: data,
        })
        .map(handlebars);

    //GET /room/{}
    let tsys2 = tsys.clone();
    let room = warp::path("room")
        .and(warp::path::param())
        .map(move |room_id: String| WithTemplate {
            name: "base",
            value: RoomData { room_id },
        })
        .map(move |with_template| render(with_template, tsys2.clone()));

    // *.js/*.css
    let bundle_js =
        warp::path("bundle.js").and(warp::fs::file(tsys.template_dir().join("bundle.js")));
    let bundle_css =
        warp::path("bundle.css").and(warp::fs::file(tsys.template_dir().join("bundle.css")));

    let route = warp::get().and(index.or(bundle_js).or(bundle_css).or(room));

    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
