//! Sensor page.

use crate::prelude::*;
use crate::templates::{self, DATE_FORMAT};
use chrono::{DateTime, Local};
use serde_json::json;

markup::define! {
    Sensor(last: Message, readings: Vec<Message>) {
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

        section.section {
            div.container {
                h3.title."is-5" { "Danger zone" }

                div.message."is-danger" {
                    div."message-body" {
                        div.content {
                            p {
                                "All readings from this sensor will get the specified sensor."
                            }
                            p {
                                "You may need that if you renamed a service and you want to move old readings to the new sensor."
                            }
                        }

                        form {
                            div.field."has-addons" {
                                div.control."is-expanded" {
                                    input[class = "input", type = "text", value = {&last.sensor}, placeholder = "Sensor"];
                                }
                                div.control {
                                    a.button."is-danger" { "Rename" }
                                }
                            }
                        }
                    }
                }

                div.message."is-danger" {
                    div."message-body" {
                        div.content {
                            p {
                                "This will " strong { "permanently delete" } " all the sensor readings."
                            }
                        }

                        form {
                            div.field {
                                div.control {
                                    button.button."is-danger" { "Delete" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
