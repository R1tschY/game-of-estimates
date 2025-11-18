use crate::web::headers::{ETag, HeaderMapExt, IfNoneMatch};
use axum::body::Body;
use axum::extract::{Query, Request};
use axum::response::{IntoResponse, Response};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{encoded_len, Engine};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};
use http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG};
use http::{HeaderValue, StatusCode};
use rust_embed::{Embed, EmbeddedFile, Metadata};
use serde::Deserialize;
use serde_json::Value;
use std::marker::PhantomData;

const ETAG_LEN: usize = match encoded_len(32, false) {
    Some(res) => res,
    // unreachable
    None => 0,
};

fn gen_etag_header(metadata: &Metadata) -> HeaderValue {
    let mut res = String::with_capacity(ETAG_LEN + 2);
    res.push('"');
    URL_SAFE_NO_PAD.encode_string(metadata.sha256_hash(), &mut res);
    res.push('"');
    HeaderValue::try_from(res).expect("invalid etag generated")
}

fn gen_etag(asset: &EmbeddedFile) -> ETag<'static> {
    ETag::new_strong(
        URL_SAFE_NO_PAD
            .encode(asset.metadata.sha256_hash())
            .into_bytes(),
    )
}

fn set_headers(response: &mut Response, metadata: &Metadata, #[allow(unused)] cache_busting: bool) {
    let headers = response.headers_mut();
    headers.insert(ETAG, gen_etag_header(metadata));
    if cache_busting {
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=604800, immutable"),
        );
    }
    headers.insert(
        CONTENT_TYPE,
        // TODO: cache content-type header values
        HeaderValue::from_str(metadata.mimetype())
            .expect("embed content-type should be valid header value"),
    );
}

pub struct AssetCatalog<T> {
    ignored: PhantomData<fn(T)>,
}

impl<T> Clone for AssetCatalog<T> {
    fn clone(&self) -> Self {
        Self {
            ignored: Default::default(),
        }
    }
}

impl<T> AssetCatalog<T> {
    pub fn new() -> Self {
        Self {
            ignored: PhantomData,
        }
    }
}

impl<T> Default for AssetCatalog<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct AssetQuery {
    v: String,
}

impl<T: Embed> AssetCatalog<T> {
    pub fn serve(&self, request: &Request) -> Response {
        let path = request.uri().path().trim_start_matches('/');
        if let Some(asset) = T::get(path) {
            let v = if let Ok(Query(AssetQuery { v })) = Query::try_from_uri(request.uri()) {
                if let Ok(v) = URL_SAFE_NO_PAD.decode(v) {
                    if &asset.metadata.sha256_hash() as &[u8] == v {
                        Some(v)
                    } else {
                        return (StatusCode::NOT_FOUND, "Version not found").into_response();
                    }
                } else {
                    return (StatusCode::BAD_REQUEST, "Invalid version query parameter")
                        .into_response();
                }
            } else {
                None
            };

            let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
            if let Some(if_none_match) = if_none_match {
                if !if_none_match.precondition_passes(&gen_etag(&asset)) {
                    let mut response = ().into_response();
                    set_headers(&mut response, &asset.metadata, v.is_some());
                    *response.status_mut() = StatusCode::NOT_MODIFIED;
                    return response;
                }
            }
            let mut response = Body::from(asset.data).into_response();
            set_headers(&mut response, &asset.metadata, v.is_some());
            response
        } else {
            (StatusCode::NOT_FOUND, "Asset not found").into_response()
        }
    }
}

// fn path<'t>(segments: Segments<'t, Path>) -> String {
//     // TODO: detect slash in segments
//     // TODO: detect .. and . and // in path
//     let res: Vec<&'t str> = segments.collect();
//     res.join("/")
// }

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
