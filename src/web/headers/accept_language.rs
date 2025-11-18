use crate::web::headers::common::QValue;
use crate::web::i18n::LanguageNegotiator;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::header::ACCEPT_LANGUAGE;
use serde::{Serialize, Serializer};
use std::convert::Infallible;
use std::str::FromStr;

pub struct Language(&'static str);

impl Language {
    #[allow(unused)]
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0)
    }
}

impl<S> FromRequestParts<S> for Language
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(request: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let accepted = request
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| AcceptLanguage::from_str(value).ok());

        let negotiator = request
            .extensions
            .get::<LanguageNegotiator>()
            .expect("LanguageNegotiator should be provided as axum extension");

        Ok(Language(negotiator.get_language(accepted)))
    }
}

pub struct AcceptLanguage {
    languages: Vec<String>,
}

impl AcceptLanguage {
    pub fn languages(&self) -> &[String] {
        &self.languages
    }
}

pub struct AcceptLanguageRejection;

impl IntoResponse for AcceptLanguageRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            "Failed to parse accept-language header",
        )
            .into_response()
    }
}

impl<S> FromRequestParts<S> for AcceptLanguage
where
    S: Send + Sync,
{
    type Rejection = AcceptLanguageRejection;

    async fn from_request_parts(request: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        match request
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
        {
            None => Err(AcceptLanguageRejection),
            Some(value) => AcceptLanguage::from_str(value).map_err(|_| AcceptLanguageRejection),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for AcceptLanguage
where
    S: Send + Sync,
{
    type Rejection = AcceptLanguageRejection;

    async fn from_request_parts(
        request: &mut Parts,
        _: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match request
            .headers
            .get(ACCEPT_LANGUAGE)
            .map(|value| value.to_str().ok())
        {
            None => Ok(None),
            Some(None) => Err(AcceptLanguageRejection),
            Some(Some(value)) => match AcceptLanguage::from_str(value) {
                Ok(res) => Ok(Some(res)),
                Err(_) => Err(AcceptLanguageRejection),
            },
        }
    }
}

impl FromStr for AcceptLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut langs = s
            .split(',')
            .map(|elem| {
                let x = elem.trim_matches(|c: char| matches!(c, ' ' | '\x09'));
                Ok(match x.split_once(';') {
                    Some((lang, weight)) => (
                        lang,
                        match QValue::parse_weight(weight.as_bytes()) {
                            Some((_, q)) => q,
                            None => return Err(()),
                        },
                    ),
                    None => (x, QValue::max()),
                })
            })
            .collect::<Result<Vec<(&str, QValue)>, Self::Err>>()?;

        langs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok(AcceptLanguage {
            languages: langs.iter().map(|x| x.0.to_string()).collect(),
        })
    }
}
