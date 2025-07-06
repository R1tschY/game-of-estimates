use handlebars::{Handlebars, JsonValue};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{error_, response, Request};
use std::borrow::Cow;

pub struct Context {
    hbs: Handlebars<'static>,
}

#[derive(Debug)]
pub struct Template {
    content_type: ContentType,
    name: Cow<'static, str>,
    value: JsonValue,
}

impl Template {
    pub fn state(hbs: Handlebars<'static>) -> Context {
        Context { hbs }
    }

    pub fn html(name: impl Into<Cow<'static, str>>, value: JsonValue) -> Self {
        Self {
            name: name.into(),
            value,
            content_type: ContentType::HTML,
        }
    }

    pub fn render(self, ctx: &Context) -> Result<(ContentType, String), Status> {
        ctx.hbs
            .render_with_context(self.name.as_ref(), &handlebars::Context::from(self.value))
            .map_err(|err| {
                error_!("Template '{}' failed to render: {}", self.name, err);
                Status::InternalServerError
            })
            .map(|out| (self.content_type, out))
    }
}

impl<'r> Responder<'r, 'static> for Template {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let ctxt = req.rocket().state::<Context>().ok_or_else(|| {
            error_!("Missing template context");
            Status::InternalServerError
        })?;

        self.render(ctxt)?.respond_to(req)
    }
}
