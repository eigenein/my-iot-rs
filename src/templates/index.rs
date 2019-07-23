//! Home page.

use crate::templates;

markup::define! {
    Index(measurements: Vec<crate::measurement::Measurement>) {
        section.hero."is-info" {
            div."hero-head" { {templates::navbar::NavBar {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4" {
                        "Dashboard"
                    }
                    h2.subtitle."is-6" {
                        {measurements.len()} " sensors"
                    }
                }
            }
        }
        section.section {
            div.container {
                div.columns."is-multiline" {
                    @for measurement in {measurements} {
                        {templates::measurement::Measurement { measurement }}
                    }
                }
            }
        }
    }
}
