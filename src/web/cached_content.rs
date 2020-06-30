use rocket::http::Status;
use rocket::response::{Content, Responder};
use rocket::{Request, Response};

pub struct Cached<R = &'static [u8]>(pub u32, pub Content<R>);

impl<'r, R: Responder<'r>> Responder<'r> for Cached<R> {
    #[inline(always)]
    fn respond_to(self, request: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .merge(self.1.respond_to(request)?)
            .raw_header("Cache-Control", format!("public, max-age={}, immutable", self.0))
            .ok()
    }
}
