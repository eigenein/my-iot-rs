//! Renders single measurement on the sensors page.

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
                        {&measurement.timestamp.format("%b %d, %H:%M:%S").to_string()}
                    }
                    p."has-text-centered"."has-text-weight-bold"[title = {format!("{:?}", &measurement.value)}] {
                        {&measurement.value}
                    }
                }
            }
        }
    }
}
