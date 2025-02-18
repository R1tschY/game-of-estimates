use crate::web::i18n::LanguageNegotiator;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Language {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let accepted = request
            .headers()
            .get_one("accept-language")
            .and_then(|value| AcceptLanguage::from_str(value).ok());

        let state = request
            .rocket()
            .state::<LanguageNegotiator>()
            .expect("AcceptLanguageHelper should be provided as state");
        Outcome::Success(Language(state.get_language(accepted)))
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AcceptLanguage {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("accept-language") {
            None => Outcome::Error((Status::BadRequest, ())),
            Some(value) => match AcceptLanguage::from_str(value) {
                Ok(res) => Outcome::Success(res),
                Err(_) => Outcome::Error((Status::BadRequest, ())),
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
                        match f32::from_str(
                            weight
                                .trim_matches(|c: char| matches!(c, ' ' | '\x09'))
                                .strip_prefix("q=")
                                .ok_or(())?,
                        ) {
                            Ok(w) => w,
                            Err(_) => return Err(()),
                        },
                    ),
                    None => (x, 1.0f32),
                })
            })
            .collect::<Result<Vec<(&str, f32)>, Self::Err>>()?;

        langs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok(AcceptLanguage {
            languages: langs.iter().map(|x| x.0.to_string()).collect(),
        })
    }
}
