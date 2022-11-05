use std::collections::HashMap;
use std::future::Future;
use std::path::Path;

use futures_util::future;
use include_dir::{include_dir, Dir, DirEntry, File};
use mime_guess::mime;
use sha1::{Digest, Sha1};
use warp::http::header::{CONTENT_TYPE, ETAG};
use warp::http::HeaderValue;
use warp::hyper::Body;
use warp::reply::Response;
use warp::{reject, Filter, Rejection, Reply};

static ASSETS_ROOT: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/public");

pub fn get_asset_content<S: AsRef<Path>>(path: S) -> Option<&'static [u8]> {
    ASSETS_ROOT.get_file(path).map(|f| f.contents())
}

// #[derive(Debug)]
// struct Conditionals {
//     if_modified_since: Option<IfModifiedSince>,
//     if_unmodified_since: Option<IfUnmodifiedSince>,
//     if_range: Option<IfRange>,
//     range: Option<Range>,
// }

pub struct AssetCatalog {
    assets: HashMap<&'static str, Asset>,
}

impl AssetCatalog {
    pub fn new() -> Self {
        let mut assets: HashMap<&'static str, Asset> = HashMap::new();
        Self::build_catalog(&ASSETS_ROOT, &mut assets);
        Self { assets }
    }

    fn build_catalog(dir: &'static Dir, assets: &mut HashMap<&'static str, Asset>) {
        for entry in dir.entries() {
            match entry {
                DirEntry::Dir(dir) => Self::build_catalog(dir, assets),
                DirEntry::File(file) => {
                    assets.insert(file.path().to_str().unwrap(), Self::process_file(file));
                }
            }
        }
    }

    fn process_file(file: &'static File) -> Asset {
        let guessed_mime = mime_guess::from_path(file.path()).first_or_octet_stream();

        let content_type = if guessed_mime.type_() == mime::TEXT
            || guessed_mime == mime::APPLICATION_JAVASCRIPT
            || guessed_mime == mime::APPLICATION_JSON
        {
            format!("{};charset=UTF-8", guessed_mime.essence_str())
        } else {
            guessed_mime.essence_str().to_string()
        };

        Asset {
            sha1: hex::encode(Sha1::digest(file.contents()))
                .into_boxed_str()
                .into(),
            content_type: content_type.into_boxed_str().into(),
            content: file.contents(),
        }
    }

    pub fn get(&self, path: &str) -> Option<&Asset> {
        self.assets.get(path)
    }
}

pub struct Asset {
    sha1: String,
    content_type: String,
    content: &'static [u8],
}

impl Reply for &'static Asset {
    fn into_response(self) -> Response {
        let mut response = Response::new(Body::from(self.content));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static(&self.content_type));
        response
            .headers_mut()
            .insert(ETAG, HeaderValue::from_static(&self.sha1));
        response
    }
}

// fn conditionals() -> impl Filter<Extract = (Conditionals,), Error = Infallible> + Copy {
//     warp::header::optional(IfModifiedSince::name().as_str())
//         .and(warp::header::optional())
//         .and(warp::header::optional())
//         .and(warp::header::optional())
//         .map(
//             |if_modified_since, if_unmodified_since, if_range, range| Conditionals {
//                 if_modified_since,
//                 if_unmodified_since,
//                 if_range,
//                 range,
//             },
//         )
// }

// fn typed_header<T>() -> impl Filter<Extract = (Option<T>,), Error = Infallible> + Copy
// where
//     T: Header + Send + 'static,
// {
//     filter_fn_one(move |route| future::ready(Ok(route.headers().typed_get())))
// }

fn asset_from_tail_path(
    tail: warp::path::Tail,
    assets: &'static AssetCatalog,
) -> impl Future<Output = Result<&'static Asset, Rejection>> + Send {
    if let Some(asset) = assets.get(tail.as_str()) {
        future::ok(asset)
    } else {
        if let Some(asset) = assets.get("index.html") {
            future::ok(asset)
        } else {
            future::err(reject::not_found())
        }
    }
}

pub fn assets(
    assets: &'static AssetCatalog,
) -> impl Filter<Extract = (&'static Asset,), Error = Rejection> + Clone {
    warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        .and(warp::any().map(move || assets))
        // .and(conditionals())
        .and_then(asset_from_tail_path)
}
