use crate::web::headers::AcceptLanguage;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Translation {
    #[allow(unused)]
    language: String,
}

impl Translation {
    #[allow(unused)]
    pub fn new(language: String) -> Self {
        Self { language }
    }

    #[allow(unused)]
    pub fn language(&self) -> &str {
        &self.language
    }
}

#[derive(Clone)]
pub struct LanguageNegotiator {
    translations: &'static [&'static str],
}

impl LanguageNegotiator {
    pub fn new(translations: &'static [&'static str]) -> Self {
        Self { translations }
    }

    pub fn get_language(&self, languages: Option<AcceptLanguage>) -> &'static str {
        languages
            .and_then(|langs| {
                langs.languages().iter().find_map(|lang| {
                    self.translations
                        .iter()
                        .copied()
                        .find(|&trans| trans == lang)
                })
            })
            .unwrap_or("en")
    }
}
