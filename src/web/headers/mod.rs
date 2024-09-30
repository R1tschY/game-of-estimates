mod parse;

mod accept_encoding;
mod accept_language;
mod common;
mod etag;

pub use accept_encoding::{AcceptEncoding, Coding};
pub use accept_language::{AcceptLanguage, Language};
pub use etag::{ETag, IfNoneMatch};

pub(crate) fn single<I: Iterator>(mut iter: I) -> Result<I::Item, InvalidHeaderValue> {
    let first = iter.next().ok_or(InvalidHeaderValue)?;
    match iter.next() {
        Some(_) => Err(InvalidHeaderValue),
        None => Ok(first),
    }
}

pub struct InvalidHeaderValue;

pub trait Header<'h> {
    fn name() -> &'static str;

    fn decode<I>(values: &mut I) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized,
        I: Iterator<Item = &'h str>;
}

/// An extension trait adding typed header methods to `rocket::http::HeaderMap`
pub trait HeaderMapExt<'h>: sealed::Sealed {
    /// Tries to decode header into `H`.
    fn typed_get<H>(&self) -> Option<H>
    where
        H: Header<'h>;

    /// Tries to decode header into `H`.
    fn typed_try_get<H>(&self) -> Result<Option<H>, InvalidHeaderValue>
    where
        H: Header<'h>;
}

impl<'h, 't> HeaderMapExt<'h> for &'h rocket::http::HeaderMap<'t> {
    fn typed_get<H>(&self) -> Option<H>
    where
        H: Header<'h>,
    {
        self.typed_try_get().unwrap_or(None)
    }

    fn typed_try_get<H>(&self) -> Result<Option<H>, InvalidHeaderValue>
    where
        H: Header<'h>,
    {
        let mut values = self.get(H::name());
        if values.size_hint().1 == Some(0) {
            Ok(None)
        } else {
            H::decode(&mut values).map(Some)
        }
    }
}

mod sealed {
    pub trait Sealed {}
    impl<'t, 'h> Sealed for &'t ::rocket::http::HeaderMap<'h> {}
}
