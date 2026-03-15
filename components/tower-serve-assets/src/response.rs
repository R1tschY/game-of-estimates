use crate::{AssertResponseModifier, Asset, gen_etag_header};
use bytes::Bytes;
use http::HeaderValue;
use http::header::ALLOW;
use http_body_util::Full;
use log::error;
use std::convert::Infallible;
use std::future::{Ready, ready};

pub(crate) type Response = http::Response<ResponseBody>;
pub(crate) type ResponseBody = Full<Bytes>;
pub type ResponseFuture = Ready<Result<http::Response<ResponseBody>, Infallible>>;

fn headers<A: Asset>(asset: &A) -> http::response::Builder {
    http::Response::builder()
        .header(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str(asset.mimetype()).unwrap(),
        )
        .header(http::header::CONTENT_LENGTH, asset.len())
        .header(http::header::ETAG, gen_etag_header(asset))
}

pub(crate) fn head<A: Asset>(
    asset: &A,
    path: &str,
    f: &impl AssertResponseModifier,
) -> ResponseFuture {
    let response = f.modify(path, headers(asset));
    to_response(response.body(empty_body()))
}

pub(crate) fn not_modified<A: Asset>(
    asset: &A,
    path: &str,
    f: impl AssertResponseModifier,
) -> ResponseFuture {
    let response = f.modify(path, headers(asset).status(http::StatusCode::NOT_MODIFIED));
    to_response(response.body(empty_body()))
}

pub(crate) fn get<A: Asset>(
    asset: A,
    path: &str,
    f: &impl AssertResponseModifier,
) -> ResponseFuture {
    let response = f.modify(path, headers(&asset));
    to_response(response.body(Full::new(asset.data())))
}

pub(crate) fn not_found() -> ResponseFuture {
    to_response(err_response(http::StatusCode::NOT_FOUND, |resp| resp))
}

pub(crate) fn method_not_allowed() -> ResponseFuture {
    to_response(err_response(http::StatusCode::NOT_FOUND, |resp| {
        resp.header(ALLOW, HeaderValue::from_static("GET,HEAD"))
    }))
}

#[inline(never)]
fn err_response(
    status_code: http::StatusCode,
    modifier: impl Fn(http::response::Builder) -> http::response::Builder,
) -> Result<Response, http::Error> {
    modifier(http::Response::builder().status(status_code)).body(empty_body())
}

fn empty_body() -> ResponseBody {
    Full::new(Bytes::new())
}

fn to_response(response: Result<Response, http::Error>) -> ResponseFuture {
    match response {
        Ok(response) => ready(Ok(response)),
        Err(err) => {
            error!("Constructed invalid HTTP response: {}", err);
            ready(Ok(http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(empty_body())
                .unwrap()))
        }
    }
}
