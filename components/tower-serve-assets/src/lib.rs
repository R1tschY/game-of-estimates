use crate::response::{ResponseBody, ResponseFuture, get, head, method_not_allowed, not_found};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{Engine, encoded_len};
use bytes::Bytes;
use http::{HeaderValue, Method, Request};
use std::borrow::Cow;
use std::convert::Infallible;
use std::task::{Context, Poll};
use tower_service::Service;

#[cfg(feature = "rust-embed")]
pub mod embed;

mod etag;
mod headers;
mod response;

const ETAG_LEN: usize = match encoded_len(32, false) {
    Some(res) => res,
    // unreachable
    None => 0,
};

pub trait Asset {
    fn data(self) -> Bytes;
    fn len(&self) -> usize;
    fn last_modified(&self) -> Option<u64>;
    fn sha256(&self) -> [u8; 32];
    fn mimetype(&self) -> &str;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait AssetCatalog {
    type Asset: Asset;

    fn get(&self, path: &str) -> Option<Self::Asset>;
    fn iter(&self) -> impl Iterator<Item = Cow<'static, str>> + 'static;
}

pub trait AssertResponseModifier {
    fn modify(&self, path: &str, response: http::response::Builder) -> http::response::Builder;
}

impl AssertResponseModifier for () {
    fn modify(&self, _: &str, response: http::response::Builder) -> http::response::Builder {
        response
    }
}

fn gen_etag_header(file: &impl Asset) -> HeaderValue {
    let mut res = String::with_capacity(ETAG_LEN + 2);
    res.push('"');
    URL_SAFE_NO_PAD.encode_string(file.sha256(), &mut res);
    res.push('"');
    HeaderValue::try_from(res).expect("invalid etag generated")
}

// fn gen_etag(file: &impl Asset) -> ETag<'static> {
//     ETag::new_strong(URL_SAFE_NO_PAD.encode(file.sha256()).into_bytes())
// }

// fn set_headers(response: &mut Response, metadata: &Metadata, cache_busting: bool) {
//     let headers = response.headers_mut();
//     headers.insert(ETAG, gen_etag_header(metadata));
//     if cache_busting {
//         headers.insert(
//             CACHE_CONTROL,
//             HeaderValue::from_static("public, max-age=604800, immutable"),
//         );
//     }
//     headers.insert(
//         CONTENT_TYPE,
//         // TODO: cache content-type header values
//         HeaderValue::from_str(metadata.mimetype())
//             .expect("embed content-type should be valid header value"),
//     );
// }

pub struct ServeAssetsBuilder<T, F> {
    catalog: T,
    index_files: Cow<'static, [Cow<'static, str>]>,
    try_suffixes: Cow<'static, [Cow<'static, str>]>,
    fallback: Option<Cow<'static, str>>,
    head_modifier: F,
}

impl<T: AssetCatalog + Clone, F> ServeAssetsBuilder<T, F> {
    pub(crate) fn new(catalog: T, head_modifier: F) -> Self {
        Self {
            catalog,
            index_files: Cow::Borrowed(&[Cow::Borrowed("index.html")]),
            try_suffixes: Cow::Borrowed(&[Cow::Borrowed(".html")]),
            fallback: None,
            head_modifier,
        }
    }

    pub fn index_files(
        mut self,
        index_files: impl Into<Cow<'static, [Cow<'static, str>]>>,
    ) -> Self {
        self.index_files = index_files.into();
        self
    }

    pub fn try_suffixes(
        mut self,
        try_suffixes: impl Into<Cow<'static, [Cow<'static, str>]>>,
    ) -> Self {
        self.try_suffixes = try_suffixes.into();
        self
    }

    pub fn fallback(mut self, fallback: impl Into<Cow<'static, str>>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    pub fn build(self) -> ServeAssets<T, F> {
        ServeAssets {
            catalog: self.catalog,
            index_files: self.index_files,
            try_suffixes: self.try_suffixes,
            fallback: self.fallback,
            head_modifier: self.head_modifier,
        }
    }
}

impl<T: AssetCatalog + Clone, F: AssertResponseModifier> ServeAssetsBuilder<T, F> {
    pub fn head_modifier(self, head_modifier: F) -> ServeAssetsBuilder<T, F> {
        ServeAssetsBuilder {
            catalog: self.catalog,
            index_files: self.index_files,
            try_suffixes: self.try_suffixes,
            fallback: self.fallback,
            head_modifier,
        }
    }
}

#[derive(Clone)]
pub struct ServeAssets<T, F> {
    catalog: T,
    index_files: Cow<'static, [Cow<'static, str>]>,
    try_suffixes: Cow<'static, [Cow<'static, str>]>,
    fallback: Option<Cow<'static, str>>,
    head_modifier: F,
}

impl<T: AssetCatalog + Clone> ServeAssets<T, ()> {
    pub fn builder(catalog: T) -> ServeAssetsBuilder<T, ()> {
        ServeAssetsBuilder::new(catalog, ())
    }
}

impl<T, ReqBody, F> Service<http::Request<ReqBody>> for ServeAssets<T, F>
where
    T: AssetCatalog + Clone,
    F: AssertResponseModifier,
{
    type Response = http::Response<ResponseBody>;
    type Error = Infallible;
    type Future = ResponseFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let method = request.method();
        if method != Method::GET && method != Method::HEAD {
            return method_not_allowed();
        }

        let path = request.uri().path();
        let path = path.strip_prefix("/").unwrap_or(path);
        let asset = if path.is_empty() {
            self.index_files
                .iter()
                .find_map(|file| self.catalog.get(file))
        } else if path.ends_with("/") {
            self.index_files
                .iter()
                .find_map(|file| self.catalog.get(&format!("{}{}", path, file)))
        } else {
            self.catalog.get(path).or_else(|| {
                self.try_suffixes
                    .iter()
                    .find_map(|suffix| self.catalog.get(&format!("{}{}", path, suffix)))
            })
        };

        let asset = if let Some(asset) = asset {
            Some(asset)
        } else if let Some(fallback) = &self.fallback {
            self.catalog.get(fallback)
        } else {
            None
        };

        if let Some(asset) = asset {
            if method == Method::HEAD {
                head(&asset, path, &self.head_modifier)
            } else {
                get(asset, path, &self.head_modifier)
            }
        } else {
            not_found()
        }
    }
}

impl<T, F> ServeAssets<T, F> {
    // fn serve_file(path: &str, request: &Request) -> Option<Response> {
    //     if let Some(asset) = T::get(path) {
    //         let immutable = path.starts_with("_app/immutable/");
    //
    //         // Check If-None-Match's E-Tag
    //         let if_none_match: Option<IfNoneMatch> = request.headers().typed_get();
    //         if let Some(if_none_match) = if_none_match {
    //             if !if_none_match.precondition_passes(&gen_etag(&asset)) {
    //                 let mut response = ().into_response();
    //                 set_headers(&mut response, &asset.metadata, immutable);
    //                 *response.status_mut() = StatusCode::NOT_MODIFIED;
    //                 return Some(response.into_response());
    //             }
    //         }
    //
    //         let mut response = Body::from(asset.data).into_response();
    //         set_headers(&mut response, &asset.metadata, immutable);
    //         Some(response)
    //     } else {
    //         None
    //     }
    // }
}
