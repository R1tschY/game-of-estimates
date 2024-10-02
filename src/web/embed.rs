use crate::web::headers::{ETag, HeaderMapExt, IfNoneMatch};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{encoded_len, Engine};
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::http::{Method, Status};
use rocket::response::{Builder, Responder};
use rocket::route::{Handler, Outcome};
use rocket::{response, Data, Request, Response, Route};
use rust_embed::{Embed, EmbeddedFile};
use std::io::Cursor;
use std::marker::PhantomData;

pub struct Asset(EmbeddedFile);

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

fn response_builder(asset: &EmbeddedFile) -> Builder<'static> {
    let mut res = Response::build();
    res.raw_header("ETag", gen_etag_header(asset));
    // TODO: make configurable: add immutable
    #[cfg(not(debug_assertions))]
    res.raw_header("Cache-Control", "public, max-age=604800");
    res.raw_header("Content-Type", asset.metadata.mimetype().to_string());
    res
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Asset {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let asset = self.0;
        Ok(response_builder(&asset)
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
        Self::new()
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
            let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
            if let Some(if_none_match) = if_none_match {
                if !if_none_match.precondition_passes(&gen_etag(&asset)) {
                    return Outcome::Success(
                        response_builder(&asset)
                            .status(Status::NotModified)
                            .finalize(),
                    );
                }
            }
            Outcome::from(request, Asset(asset))
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

    pub fn get(file_path: &str) -> Option<Asset> {
        T::get(file_path).map(Asset)
    }
}

fn path<'t>(segments: Segments<'t, Path>) -> String {
    // TODO: detect slash in segments
    // TODO: detect .. and . and // in path
    let res: Vec<&'t str> = segments.collect();
    res.join("/")
}
