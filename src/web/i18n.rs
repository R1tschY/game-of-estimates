use crate::web::headers::AcceptLanguage;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
};
use rocket::serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, Clone)]
pub struct Translation {
    #[allow(unused)]
    language: String,
    strings: HashMap<String, String>,
}

impl Translation {
    #[allow(unused)]
    pub fn new(language: String, strings: HashMap<String, String>) -> Self {
        Self { language, strings }
    }

    #[allow(unused)]
    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn strings(&self) -> &HashMap<String, String> {
        &self.strings
    }
}

#[derive(Clone)]
pub struct AcceptLanguageHelper {
    translations: Arc<Vec<&'static str>>,
}

impl AcceptLanguageHelper {
    pub fn new(translations: &HashMap<&'static str, Translation>) -> Self {
        Self {
            translations: Arc::new(translations.iter().map(|x| *x.0).collect()),
        }
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

#[derive(Clone)]
pub struct I18nHelper {
    translations: HashMap<&'static str, Translation>,
}

impl I18nHelper {
    pub fn new(translations: HashMap<&'static str, Translation>) -> Self {
        Self { translations }
    }
}

impl HelperDef for I18nHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        helper: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        let arg = match helper.param(0) {
            None => Err(RenderError::from(RenderErrorReason::ParamNotFoundForIndex(
                "text", 0,
            ))),
            Some(param) => match param.value() {
                Value::String(param) => Ok(param),
                _ => Err(RenderError::from(RenderErrorReason::Other(
                    "Helper text param at index 0 has invalid param type, string expected"
                        .to_string(),
                ))),
            },
        }?;

        let lang = match ctx.data() {
            Value::Object(ctx) => match ctx.get("lang") {
                Some(Value::String(lang)) => Ok(lang),
                _ => Err(RenderError::from(RenderErrorReason::Other(
                    "Helper text ctx param lang required".to_string(),
                ))),
            },
            _ => Err(RenderError::from(RenderErrorReason::Other(
                "Helper text requires context to be an object".to_string(),
            ))),
        }?;

        let translation = self.translations.get(lang.as_str()).unwrap();
        let text = translation.strings().get(arg).ok_or_else(|| {
            RenderError::from(RenderErrorReason::Other(format!(
                "Helper text argument '{}' is not a known string for {} translation",
                arg, "de"
            )))
        })?;

        Ok(Value::String(text.to_owned()).into())
    }
}
