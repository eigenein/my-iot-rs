use crate::prelude::*;
use rocket::http::hyper::header::EntityTag;

impl Reading {
    pub fn entity_tag(&self) -> EntityTag {
        EntityTag::new(true, format!("{:x}", self.timestamp.timestamp_millis()))
    }
}
