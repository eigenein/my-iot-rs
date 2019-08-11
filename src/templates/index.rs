//! Home page.

use crate::templates;

markup::define! {
    Index(readings: Vec<crate::reading::Reading>) {
        section.hero."is-info" {
            div."hero-head" { {templates::navbar::NavBar {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4" {
                        "Dashboard"
                    }
                    h2.subtitle."is-6" {
                        {readings.len()} " sensors"
                    }
                }
            }
        }
        section.section {
            div.container {
                div.columns."is-multiline" {
                    @for reading in {readings} {
                        {templates::reading::Reading { reading }}
                    }
                }
            }
        }
    }
}
