use crate::web::headers::{ETag, HeaderMapExt, IfNoneMatch};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{encoded_len, Engine};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::http::{Method, Status};
use rocket::response::{Builder, Responder};
use rocket::route::{Handler, Outcome};
use rocket::{response, Data, Request, Response, Route};
use rust_embed::{Embed, EmbeddedFile};
use serde_json::Value;
use std::io::Cursor;
use std::marker::PhantomData;

pub struct Asset {
    file: EmbeddedFile,
    cache_busting: bool,
}

const ETAG_LEN: usize = match encoded_len(32, false) {
    Some(res) => res,
    // unreachable
    None => 0,
};

fn gen_etag_header(asset: &EmbeddedFile) -> String {
    let mut res = String::with_capacity(ETAG_LEN + 2);
    res.push('"');
    URL_SAFE_NO_PAD.encode_string(asset.metadata.sha256_hash(), &mut res);
    res.push('"');
    res
}

fn gen_etag(asset: &EmbeddedFile) -> ETag<'static> {
    ETag::new_strong(URL_SAFE_NO_PAD.encode(asset.metadata.sha256_hash()))
}

fn response_builder(
    asset: &EmbeddedFile,
    #[allow(unused)] cache_busting: bool,
) -> Builder<'static> {
    let mut res = Response::build();
    res.raw_header("ETag", gen_etag_header(asset));
    #[cfg(not(debug_assertions))]
    {
        if cache_busting {
            res.raw_header("Cache-Control", "public, max-age=604800, immutable");
        }
    }
    res.raw_header("Content-Type", asset.metadata.mimetype().to_string());
    res
}

impl<'r> Responder<'r, 'static> for Asset {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let asset = self.file;
        Ok(response_builder(&asset, self.cache_busting)
            .sized_body(asset.data.len(), Cursor::new(asset.data))
            .finalize())
    }
}

pub struct AssetCatalog<T: Embed> {
    ignored: PhantomData<fn(T)>,
    rank: isize,
}

impl<T: Embed> Clone for AssetCatalog<T> {
    fn clone(&self) -> Self {
        Self {
            ignored: Default::default(),
            rank: self.rank,
        }
    }
}

impl<T: Embed> AssetCatalog<T> {
    pub fn new() -> Self {
        Self {
            ignored: PhantomData,
            rank: Self::DEFAULT_RANK,
        }
    }
}

impl<T: Embed> Default for AssetCatalog<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[rocket::async_trait]
impl<T: Embed + 'static> Handler for AssetCatalog<T> {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let path = path(request.routed_segments(0..));
        if let Some(asset) = T::get(&path) {
            let v = if let Some(v) = request.query_value::<&str>("v") {
                if let Some(v) = v.ok().and_then(|v| URL_SAFE_NO_PAD.decode(v).ok()) {
                    if &asset.metadata.sha256_hash() as &[u8] == v {
                        Some(v)
                    } else {
                        return Outcome::forward(data, Status::NotFound);
                    }
                } else {
                    return Outcome::forward(data, Status::BadRequest);
                }
            } else {
                None
            };

            let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
            if let Some(if_none_match) = if_none_match {
                if !if_none_match.precondition_passes(&gen_etag(&asset)) {
                    return Outcome::Success(
                        response_builder(&asset, v.is_some())
                            .status(Status::NotModified)
                            .finalize(),
                    );
                }
            }
            Outcome::from(
                request,
                Asset {
                    file: asset,
                    cache_busting: v.is_some(),
                },
            )
        } else {
            Outcome::forward(data, Status::NotFound)
        }
    }
}

impl<T: Embed + 'static> From<AssetCatalog<T>> for Vec<Route> {
    fn from(value: AssetCatalog<T>) -> Self {
        let mut route = Route::ranked(10, Method::Get, "/<path..>", value);
        route.name = Some(format!("AssetCatalog: {}", std::any::type_name::<T>()).into());
        vec![route]
    }
}

impl<T: Embed> AssetCatalog<T> {
    const DEFAULT_RANK: isize = 10;

    pub fn rank(mut self, rank: isize) -> Self {
        self.rank = rank;
        self
    }

    pub fn get(file_path: &str, cache_busting: bool) -> Option<Asset> {
        T::get(file_path).map(|file| Asset {
            file,
            cache_busting,
        })
    }
}

fn path<'t>(segments: Segments<'t, Path>) -> String {
    // TODO: detect slash in segments
    // TODO: detect .. and . and // in path
    let res: Vec<&'t str> = segments.collect();
    res.join("/")
}

pub fn get_asset_url<T: Embed>(file_path: &str) -> Option<String> {
    T::get(file_path).map(|file| {
        let mut res = String::with_capacity(1 + file_path.len() + 3 + 43);
        res.push('/');
        res.push_str(file_path);
        res.push_str("?v=");
        URL_SAFE_NO_PAD.encode_string(file.metadata.sha256_hash(), &mut res);
        res
    })
}

#[derive(Clone)]
pub struct EmbedTemplateContext<T>(PhantomData<T>);

impl<T> EmbedTemplateContext<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Embed> HelperDef for EmbedTemplateContext<T> {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let arg = match helper.param(0) {
            None => Err(RenderError::from(RenderErrorReason::ParamNotFoundForIndex(
                "asset", 0,
            ))),
            Some(param) => match param.value() {
                Value::String(param) => Ok(param),
                _ => Err(RenderError::from(RenderErrorReason::Other(
                    "Helper asset param at index 0 has invalid param type, string expected"
                        .to_string(),
                ))),
            },
        }?;

        match get_asset_url::<T>(arg) {
            Some(url) => Ok(Value::String(url).into()),
            None => Err(RenderError::from(RenderErrorReason::Other(format!(
                "Helper asset argument '{arg}' is not a known asset",
            )))),
        }
    }
}
