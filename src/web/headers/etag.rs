use crate::web::headers::single;
use crate::web::headers::{Header, InvalidHeaderValue};
use http::HeaderValue;
use std::borrow::Cow;

pub struct ETag<'h>(EntityTag<'h>);

impl<'h> ETag<'h> {
    pub fn new_strong(etag: impl Into<Cow<'h, [u8]>>) -> Self {
        ETag(EntityTag {
            weak: false,
            entity_tag: etag.into(),
        })
    }

    #[allow(unused)]
    pub fn new_weak(etag: impl Into<Cow<'h, [u8]>>) -> Self {
        ETag(EntityTag {
            weak: true,
            entity_tag: etag.into(),
        })
    }
}

impl<'h> Header<'h> for ETag<'h> {
    fn name() -> &'static str {
        "ETag"
    }

    fn decode<I>(values: &mut I) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized,
        I: Iterator<Item = &'h HeaderValue>,
    {
        single(values)
            .and_then(|value| EntityTag::parse(value.as_bytes()))
            .map(ETag)
    }
}

// TODO: use smallvec
pub struct IfNoneMatch<'h>(Vec<EntityPattern<'h>>);

impl<'h> IfNoneMatch<'h> {
    pub fn precondition_passes(&self, etag: &ETag<'_>) -> bool {
        !self.0.iter().any(|e| e.matches_weak(etag))
    }
}

fn trim_start(bytes: &[u8], pred: impl Fn(&u8) -> bool) -> &[u8] {
    if let Some(i) = bytes.iter().position(|c| !pred(c)) {
        &bytes[i..bytes.len()]
    } else {
        bytes
    }
}

fn trim_end(bytes: &[u8], pred: impl Fn(&u8) -> bool) -> &[u8] {
    if let Some(i) = bytes.iter().rposition(|c| !pred(c)) {
        &bytes[0..=i]
    } else {
        bytes
    }
}

fn trim(bytes: &[u8], pred: impl Fn(&u8) -> bool) -> &[u8] {
    trim_end(trim_start(bytes, &pred), &pred)
}

impl<'h> Header<'h> for IfNoneMatch<'h> {
    fn name() -> &'static str {
        "If-None-Match"
    }

    fn decode<I>(values: &mut I) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized,
        I: Iterator<Item = &'h HeaderValue>,
    {
        values
            .flat_map(|v| v.as_bytes().split(|&c| c == b','))
            .map(|v| trim(v, |&c| c == b' ' || c == b'\t'))
            .map(EntityPattern::parse)
            .collect::<Result<Vec<EntityPattern>, InvalidHeaderValue>>()
            .map(IfNoneMatch)
    }
}

struct EntityTag<'h> {
    weak: bool,
    entity_tag: Cow<'h, [u8]>,
}

impl<'h> EntityTag<'h> {
    pub fn parse(input: &'h [u8]) -> Result<Self, InvalidHeaderValue> {
        if input.len() < 2 || !input.ends_with(b"\"") {
            return Err(InvalidHeaderValue);
        }

        if input.starts_with(b"\"") {
            Ok(EntityTag {
                weak: false,
                entity_tag: Cow::from(&input[1..input.len() - 1]),
            })
        } else if input.starts_with(b"W/\"") {
            Ok(EntityTag {
                weak: true,
                entity_tag: Cow::from(&input[3..input.len() - 1]),
            })
        } else {
            Err(InvalidHeaderValue)
        }
    }

    #[allow(unused)]
    pub fn write(&self) -> Vec<u8> {
        if self.weak {
            let mut res: Vec<u8> = Vec::with_capacity(4 + self.entity_tag.len());
            res.extend_from_slice(b"W/\"");
            res.extend_from_slice(&self.entity_tag);
            res.push(b'"');
            res
        } else {
            let mut res: Vec<u8> = Vec::with_capacity(2 + self.entity_tag.len());
            res.push(b'"');
            res.extend_from_slice(&self.entity_tag);
            res.push(b'"');
            res
        }
    }

    pub fn matches_weak(&self, etag: &EntityTag<'_>) -> bool {
        self.entity_tag == etag.entity_tag
    }

    #[allow(unused)]
    pub fn matches_strong(&self, etag: &EntityTag<'_>) -> bool {
        !self.weak && !etag.weak && self.entity_tag == etag.entity_tag
    }
}

enum EntityPattern<'h> {
    Any,
    EntityTag(EntityTag<'h>),
}

impl<'h> EntityPattern<'h> {
    pub fn parse(input: &'h [u8]) -> Result<Self, InvalidHeaderValue> {
        if input == b"*" {
            Ok(Self::Any)
        } else {
            EntityTag::parse(input).map(Self::EntityTag)
        }
    }

    #[allow(unused)]
    pub fn write(&self) -> Cow<'static, [u8]> {
        match self {
            EntityPattern::Any => b"*".into(),
            EntityPattern::EntityTag(entity_tag) => entity_tag.write().into(),
        }
    }

    pub fn matches_weak(&self, etag: &ETag<'_>) -> bool {
        match self {
            EntityPattern::Any => true,
            EntityPattern::EntityTag(e) => e.matches_weak(&etag.0),
        }
    }

    #[allow(unused)]
    pub fn matches_strong(&self, etag: &ETag<'_>) -> bool {
        match self {
            EntityPattern::Any => true,
            EntityPattern::EntityTag(e) => e.matches_strong(&etag.0),
        }
    }
}
