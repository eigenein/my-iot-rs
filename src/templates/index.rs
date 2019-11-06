//! Home page.

use crate::core::persistence::select_actuals;
use crate::prelude::*;
use crate::templates::*;

markup::define! {
    IndexTemplate(db: Arc<Mutex<Connection>>) {
        {let actuals = select_actuals(&db.lock().unwrap()).unwrap();}
        section.hero."is-info" {
            div."hero-head" { {NavBarTemplate {}} }
            div."hero-body" {
                div.container {
                    h1.title."is-4" {
                        "Dashboard"
                    }
                    h2.subtitle."is-6" {
                        {actuals.len()} " sensors"
                    }
                }
            }
        }
        section.section {
            div.container {
                div.columns."is-multiline" {
                    @for (sensor, reading) in {actuals} {
                        {ReadingTemplate { sensor: &sensor, reading: &reading }}
                    }
                }
            }
        }
    }
}
