use crate::web::headers::single;
use crate::web::headers::{Header, InvalidHeaderValue};
use std::borrow::Cow;

pub struct ETag<'h>(EntityTag<'h>);

impl<'h> ETag<'h> {
    pub fn new_strong(etag: impl Into<Cow<'h, str>>) -> Self {
        ETag(EntityTag {
            weak: false,
            entity_tag: etag.into(),
        })
    }

    #[allow(unused)]
    pub fn new_weak(etag: impl Into<Cow<'h, str>>) -> Self {
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
        I: Iterator<Item = &'h str>,
    {
        single(values).and_then(EntityTag::parse).map(ETag)
    }
}

// TODO: use smallvec
pub struct IfNoneMatch<'h>(Vec<EntityPattern<'h>>);

impl<'h> IfNoneMatch<'h> {
    pub fn precondition_passes(&self, etag: &ETag<'_>) -> bool {
        !self.0.iter().any(|e| e.matches_weak(etag))
    }
}

impl<'h> Header<'h> for IfNoneMatch<'h> {
    fn name() -> &'static str {
        "If-None-Match"
    }

    fn decode<I>(values: &mut I) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized,
        I: Iterator<Item = &'h str>,
    {
        values
            .flat_map(|v| v.split(','))
            .map(|v| v.trim_matches(|c| c == ' ' || c == '\t'))
            .map(EntityPattern::parse)
            .collect::<Result<Vec<EntityPattern>, InvalidHeaderValue>>()
            .map(IfNoneMatch)
    }
}

struct EntityTag<'h> {
    weak: bool,
    entity_tag: Cow<'h, str>,
}

impl<'h> EntityTag<'h> {
    pub fn parse(input: &'h str) -> Result<Self, InvalidHeaderValue> {
        if input.len() < 2 || !input.ends_with('"') {
            return Err(InvalidHeaderValue);
        }

        if input.starts_with('"') {
            Ok(EntityTag {
                weak: false,
                entity_tag: Cow::from(&input[1..input.len() - 1]),
            })
        } else if input.starts_with("W/\"") {
            Ok(EntityTag {
                weak: true,
                entity_tag: Cow::from(&input[3..input.len() - 1]),
            })
        } else {
            Err(InvalidHeaderValue)
        }
    }

    #[allow(unused)]
    pub fn write(&self) -> String {
        if self.weak {
            format!("W/\"{}\"", self.entity_tag)
        } else {
            format!("\"{}\"", self.entity_tag)
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
    pub fn parse(input: &'h str) -> Result<Self, InvalidHeaderValue> {
        if input == "*" {
            Ok(Self::Any)
        } else {
            EntityTag::parse(input).map(Self::EntityTag)
        }
    }

    #[allow(unused)]
    pub fn write(&self) -> Cow<'static, str> {
        match self {
            EntityPattern::Any => "*".into(),
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
