//! Renders single reading on the sensors page.

use crate::prelude::*;
use crate::templates::DATE_FORMAT;

// TODO: title should be human-readable.
markup::define! {
    ReadingTemplate<'a>(sensor: &'a Sensor, reading: &'a: Reading) {
        div."column"."is-one-quarter" {
            a[href = {format!("/sensors/{}", &sensor.sensor)} ] {
                div.notification.reading.{reading.value.class()} {
                    p.title."is-6"[title = {&sensor.sensor}] {
                        {&sensor.sensor}
                    }
                    p.subtitle."is-7"[title = {&reading.timestamp.to_string()}] {
                        {&reading.timestamp.format(DATE_FORMAT).to_string()}
                    }
                    p."has-text-centered"."has-text-weight-bold"[title = {format!("{:?}", &reading.value)}] {
                        {&reading.value}
                    }
                }
            }
        }
    }
}
