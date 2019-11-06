//! Sensor page.

use crate::prelude::*;
use crate::templates::{self, DATE_FORMAT};

markup::define! {
    SensorTemplate(sensor_id: String, reading: Reading) {
        section.hero.{&reading.value.class()} {
            div."hero-head" { {templates::NavBarTemplate {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4"[title = {format!("{:?}", &reading.value)}] {
                        @if reading.value.is_inline() {
                            {&reading.value}
                        } else {
                            {&sensor_id}
                        }
                    }
                    h2.subtitle."is-6" {
                        @if reading.value.is_inline() {
                            {&sensor_id} " "
                        }
                        span { i.far."fa-clock" {} } " "
                        span[title = {&reading.timestamp.to_string()}] {
                            {&reading.timestamp.format(DATE_FORMAT).to_string()}
                        }
                    }
                }
            }
        }

        @if !reading.value.is_inline() {
            section.section {
                div.container {
                    div.message {
                        div."message-body" {
                            {&reading.value}
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
                                {format!("{:#?}", &reading)}
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
                                    input[class = "input", type = "text", value = {&sensor_id}, placeholder = "Sensor"];
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
