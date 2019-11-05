//! Home page.

use crate::prelude::*;
use crate::templates::*;

markup::define! {
    IndexTemplate<'a>(db: &'a Arc<Mutex<SqliteConnection>>) {
        section.hero."is-info" {
            div."hero-head" { {NavBarTemplate {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4" {
                        "Dashboard"
                    }
                    h2.subtitle."is-6" {
                        {messages.len()} " sensors"
                    }
                }
            }
        }
        section.section {
            div.container {
                div.columns."is-multiline" {
                    @for message in {messages} {
                        {ReadingTemplate { sensor, reading }}
                    }
                }
            }
        }
    }
}
