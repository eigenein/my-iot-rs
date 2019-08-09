//! Status page.

use crate::templates;

markup::define! {
    Status(settings: crate::settings::Settings) {
        section.hero."is-info" {
            div."hero-head" { {templates::navbar::NavBar {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4" {
                        "Status"
                    }
                    h2.subtitle."is-6" {
                        "Extended engine status"
                    }
                }
            }
        }
        section.section {
            div.container {
                h3.title."is-5" { "Settings" }
                h4.subtitle."is-7" {
                    "This is what configured in the settings file"
                }
                div.message {
                    div."message-body" {
                        pre {
                            code {
                                {format!("{:#?}", &settings)}
                            }
                        }
                    }
                }
            }
        }
    }
}
