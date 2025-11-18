use axum::response::{IntoResponse, Response};
use handlebars::{Handlebars, JsonValue, RenderError};
use http::{header, HeaderValue, StatusCode};
use std::borrow::Cow;

pub type TemplateResult = Result<Response, TemplateRenderRejection>;

pub struct TemplateRenderer {
    hbs: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new(hbs: Handlebars<'static>) -> Self {
        Self { hbs }
    }

    pub fn render(&self, template: Template) -> TemplateResult {
        self.hbs
            .render_with_context(
                template.name.as_ref(),
                &handlebars::Context::from(template.value),
            )
            .map_err(|err| TemplateRenderRejection {
                name: template.name,
                err,
            })
            .map(|out| {
                let mut res = out.into_response();
                res.headers_mut()
                    .insert(header::CONTENT_TYPE, template.content_type);
                res
            })
    }
}

pub struct TemplateRenderRejection {
    name: Cow<'static, str>,
    err: RenderError,
}

impl IntoResponse for TemplateRenderRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template '{}' failed to render: {}", self.name, self.err),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct Template {
    content_type: HeaderValue,
    name: Cow<'static, str>,
    value: JsonValue,
}

impl Template {
    pub fn html(name: impl Into<Cow<'static, str>>, value: JsonValue) -> Self {
        Self {
            name: name.into(),
            value,
            content_type: HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        }
    }
}
