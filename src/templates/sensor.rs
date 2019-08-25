//! Sensor page.

use crate::reading::Reading;
use crate::templates;
use crate::templates::DATE_FORMAT;
use crate::value::Value;
use chrono::{DateTime, Local};
use serde_json::json;

markup::define! {
    Sensor(last: Reading, readings: Vec<Reading>) {
        section.hero.{&last.value.class()} {
            div."hero-head" { {templates::navbar::NavBar {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4"[title = {format!("{:?}", &last.value)}] {
                        @if last.value.is_inline() {
                            {&last.value}
                        } else {
                            {&last.sensor}
                        }
                    }
                    h2.subtitle."is-6" {
                        @if last.value.is_inline() {
                            {&last.sensor} " "
                        }
                        span { i.far."fa-clock" {} } " "
                        span[title = {&last.timestamp.to_string()}] {
                            {&last.timestamp.format(DATE_FORMAT).to_string()}
                        }
                    }
                }
            }
        }

        @if !last.value.is_inline() {
            section.section {
                div.container {
                    div.message {
                        div."message-body" {
                            {&last.value}
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

        section.section {
            div.container {
                h3.title."is-5" { "JSON" }
                div.message {
                    div."message-body" {
                        pre {
                            code {
                                {
                                    let (xs, ys): (Vec<DateTime<Local>>, Vec<Value>) = readings
                                        .iter()
                                        .map(|reading| (reading.timestamp, reading.value.clone()))
                                        .unzip();
                                }
                                {
                                    serde_json::to_string(&json!({
                                        "x": xs,
                                        "y": ys,
                                    })).unwrap()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
