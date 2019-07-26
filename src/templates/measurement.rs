//! Renders single measurement on the sensors page.

use crate::templates::DATE_FORMAT;

// TODO: title should be human-readable.
markup::define! {
    Measurement<'a>(measurement: &'a crate::measurement::Measurement) {
        div."column"."is-one-quarter" {
            a[href = {format!("/sensors/{}", &measurement.sensor)} ] {
                div.notification.measurement.{measurement.value.class()} {
                    p.title."is-6"[title = {&measurement.sensor}] {
                        {&measurement.sensor}
                    }
                    p.subtitle."is-7"[title = {&measurement.timestamp.to_string()}] {
                        {&measurement.timestamp.format(DATE_FORMAT).to_string()}
                    }
                    p."has-text-centered"."has-text-weight-bold"[title = {format!("{:?}", &measurement.value)}] {
                        {&measurement.value}
                    }
                }
            }
        }
    }
}
