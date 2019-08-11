//! Sensor page.

use crate::reading::Reading;
use crate::templates;
use crate::templates::DATE_FORMAT;

markup::define! {
    Sensor(last: Reading) {
        section.hero.{&last.value.class()} {
            div."hero-head" { {templates::navbar::NavBar {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4"[title = {format!("{:?}", &last.value)}] {
                        {&last.value}
                    }
                    h2.subtitle."is-6" {
                        {&last.sensor} " "
                        span { i.far."fa-clock" {} } " "
                        span[title = {&last.timestamp.to_string()}] {
                            {&last.timestamp.format(DATE_FORMAT).to_string()}
                        }
                    }
                }
            }
        }
        section.section {
            div.container {
                h3.title."is-5" { "Latest reading" }
                h4.subtitle."is-7" { "This is what is stored in the database" }
                div.message {
                    div."message-body" {
                        pre {
                            code {
                                {format!("{:#?}", &last)}
                            }
                        }
                    }
                }
            }
        }
    }
}
