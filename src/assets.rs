use std::future::Future;
use std::path::Path;

use futures_util::future;
use include_dir::{include_dir, Dir};
use warp::hyper::Body;
use warp::reply::Response;
use warp::{reject, Filter, Rejection, Reply};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/public");

pub fn get_asset<S: AsRef<Path>>(path: S) -> Option<&'static [u8]> {
    ASSETS.get_file(path).map(|f| f.contents())
}

// #[derive(Debug)]
// struct Conditionals {
//     if_modified_since: Option<IfModifiedSince>,
//     if_unmodified_since: Option<IfUnmodifiedSince>,
//     if_range: Option<IfRange>,
//     range: Option<Range>,
// }

pub struct Asset(&'static [u8]);

impl Reply for Asset {
    fn into_response(self) -> Response {
        Response::new(Body::from(self.0))
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
) -> impl Future<Output = Result<Asset, Rejection>> + Send {
    if let Some(bytes) = get_asset(tail.as_str()) {
        future::ok(Asset(bytes))
    } else {
        if let Some(bytes) = get_asset("index.html") {
            future::ok(Asset(bytes))
        } else {
            future::err(reject::not_found())
        }
    }
}

pub fn assets() -> impl Filter<Extract = (Asset,), Error = Rejection> + Clone {
    warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        // .and(conditionals())
        .and_then(asset_from_tail_path)
}
