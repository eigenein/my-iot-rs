use rocket::http::hyper::header::EntityTag;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

/// Extracts a [`If-None-Match`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag) header
/// from a request.
pub struct IfNoneMatch(pub EntityTag);

impl<'a, 'r> FromRequest<'a, 'r> for IfNoneMatch {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request
            .headers()
            .get_one("If-None-Match")
            .and_then(|value| value.parse::<EntityTag>().ok())
        {
            Some(entity_tag) => Outcome::Success(IfNoneMatch(entity_tag)),
            None => Outcome::Forward(()),
        }
    }
}
