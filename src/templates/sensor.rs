//! Sensor page.

use crate::prelude::*;
use crate::templates::{self, DATE_FORMAT};

markup::define! {
    SensorTemplate(last: Message, readings: Vec<Message>) {
        section.hero.{&last.reading.value.class()} {
            div."hero-head" { {templates::NavBarTemplate {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4"[title = {format!("{:?}", &last.reading.value)}] {
                        @if last.reading.value.is_inline() {
                            {&last.reading.value}
                        } else {
                            {&last.sensor.sensor}
                        }
                    }
                    h2.subtitle."is-6" {
                        @if last.reading.value.is_inline() {
                            {&last.sensor.sensor} " "
                        }
                        span { i.far."fa-clock" {} } " "
                        span[title = {&last.reading.timestamp.to_string()}] {
                            {&last.reading.timestamp.format(DATE_FORMAT).to_string()}
                        }
                    }
                }
            }
        }

        @if !last.reading.value.is_inline() {
            section.section {
                div.container {
                    div.message {
                        div."message-body" {
                            {&last.reading.value}
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
                                {format!("{:#?}", &last.reading)}
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
                                "All readings from this sensor will be moved to the specified sensor."
                            }
                            p {
                                "You may need that if you renamed a service and you want to move old readings to the new sensor."
                            }
                        }

                        form {
                            div.field."has-addons" {
                                div.control."is-expanded" {
                                    input[class = "input", type = "text", value = {&last.sensor.sensor}, placeholder = "Sensor"];
                                }
                                div.control {
                                    a.button."is-danger" { "Move" }
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
