//! Web interface templates.

mod base;
mod index;
mod navbar;
mod reading;
mod sensor;

pub use base::BaseTemplate;
pub use index::IndexTemplate;
pub use navbar::NavBarTemplate;
pub use reading::ReadingTemplate;
pub use sensor::SensorTemplate;

const DATE_FORMAT: &str = "%b %d, %H:%M:%S";
