use rocket::http::Status;
use rocket::response::Responder;
use rocket::{response, Request, Response};

pub struct SeeOther {
    location: String,
}

impl SeeOther {
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for SeeOther {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(Status::SeeOther)
            .raw_header("location", self.location)
            .ok()
    }
}
