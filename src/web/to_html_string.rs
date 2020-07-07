use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::{Request, Response};

/// Responds with an HTML body for any `T: ToString`.
pub struct ToHtmlString<T>(pub T);

impl<'r, R: ToString> Responder<'r> for ToHtmlString<R> {
    #[inline(always)]
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .merge(self.0.to_string().respond_to(request)?)
            .header(ContentType::HTML)
            .raw_header("Cache-Control", "private")
            .ok()
    }
}
