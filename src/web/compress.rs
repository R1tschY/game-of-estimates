use crate::web::headers::{AcceptEncoding, Coding, Header, HeaderMapExt};
#[cfg(feature = "compress-brotli")]
use async_compression::tokio::bufread::BrotliEncoder;
#[cfg(feature = "compress-deflate")]
use async_compression::tokio::bufread::DeflateEncoder;
#[cfg(feature = "compress-gzip")]
use async_compression::tokio::bufread::GzipEncoder;
#[cfg(feature = "compress-zstd")]
use async_compression::tokio::bufread::ZstdEncoder;
use async_compression::Level;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::{response, Request, Response};
use tokio::io::BufReader;

const SUPPORTED: &[Coding] = &[
    #[cfg(feature = "compress-zstd")]
    Coding::Zstandard,
    #[cfg(feature = "compress-brotli")]
    Coding::Brotli,
    #[cfg(feature = "compress-gzip")]
    Coding::Gzip,
    #[cfg(feature = "compress-deflate")]
    Coding::Deflate,
];

fn compress(lvl: Level, request: &Request<'_>, response: &mut Response) {
    if response.headers().get_one("Content-Encoding").is_some() {
        return;
    }

    let ae = request
        .headers()
        .typed_get::<AcceptEncoding>()
        .unwrap_or_else(AcceptEncoding::default);
    match ae.match_one_of(SUPPORTED) {
        Some(coding) => {
            let mut response = Response::new();

            match coding {
                #[cfg(feature = "compress-gzip")]
                Coding::Gzip => {
                    let body =
                        GzipEncoder::with_quality(BufReader::new(response.body_mut().take()), lvl);
                    response.set_streamed_body(body)
                }
                #[cfg(feature = "compress-deflate")]
                Coding::Deflate => {
                    let body = DeflateEncoder::with_quality(
                        BufReader::new(response.body_mut().take()),
                        lvl,
                    );
                    response.set_streamed_body(body)
                }
                #[cfg(feature = "compress-brotli")]
                Coding::Brotli => {
                    let body = BrotliEncoder::with_quality(
                        BufReader::new(response.body_mut().take()),
                        lvl,
                    );
                    response.set_streamed_body(body)
                }
                #[cfg(feature = "compress-zstd")]
                Coding::Zstandard => {
                    let body =
                        ZstdEncoder::with_quality(BufReader::new(response.body_mut().take()), lvl);
                    response.set_streamed_body(body)
                }
                Coding::Identity => return,
            };

            response.set_header(rocket::http::Header::new(
                "Content-Encoding",
                coding.as_str(),
            ));
        }
        _ => {
            *response = Response::build()
                .status(Status::NotAcceptable)
                .raw_header(
                    AcceptEncoding::name(),
                    SUPPORTED
                        .iter()
                        .map(|c| c.as_str())
                        .collect::<Vec<&str>>()
                        .join(", "),
                )
                .finalize()
        }
    }
}

pub struct Compressed<T> {
    level: Level,
    inner: T,
}

impl<'r, 'o: 'r, T: Responder<'r, 'o>> Responder<'r, 'o> for Compressed<T> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
        let mut response = self.inner.respond_to(request)?;
        compress(self.level, request, &mut response);
        Ok(response)
    }
}

pub trait CompressionPredicate {
    fn should_compress(&self, response: &Response) -> bool;

    fn and<Other>(self, other: Other) -> And<Self, Other>
    where
        Self: Sized,
        Other: CompressionPredicate,
    {
        And(self, other)
    }
}

pub struct DefaultPredicate(And<SizeAbove, NotForCompressedContentTypes>);

impl CompressionPredicate for DefaultPredicate {
    fn should_compress(&self, response: &Response) -> bool {
        self.0.should_compress(response)
    }
}

impl Default for DefaultPredicate {
    fn default() -> Self {
        Self(SizeAbove(256).and(NotForCompressedContentTypes))
    }
}

pub struct And<L, R>(L, R);

impl<L, R> CompressionPredicate for And<L, R>
where
    L: CompressionPredicate,
    R: CompressionPredicate,
{
    fn should_compress(&self, response: &Response) -> bool {
        self.0.should_compress(response) && self.1.should_compress(response)
    }
}

pub struct SizeAbove(usize);

impl CompressionPredicate for SizeAbove {
    fn should_compress(&self, response: &Response) -> bool {
        match response.body().preset_size() {
            Some(size) => size >= self.0,
            _ => true,
        }
    }
}

pub struct NotForCompressedContentTypes;

impl CompressionPredicate for NotForCompressedContentTypes {
    fn should_compress(&self, response: &Response) -> bool {
        if let Some(ct) = response.content_type() {
            ct.top() != "image" || ct.sub() != "svg+xml"
        } else {
            true
        }
    }
}

pub struct Compression<P = DefaultPredicate> {
    level: Level,
    pred: P,
}

impl<P> Compression<P> {
    pub fn new(level: Level, pred: P) -> Self {
        Self { level, pred }
    }
}

#[rocket::async_trait]
impl<P: CompressionPredicate + Sync + Send + 'static> Fairing for Compression<P> {
    fn info(&self) -> Info {
        Info {
            name: "Compress responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        if self.pred.should_compress(res) {
            compress(self.level, req, res)
        }
    }
}
