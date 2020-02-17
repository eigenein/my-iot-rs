#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    pub sensor_id: String,
    pub title: Option<String>,
}

impl Sensor {
    pub fn display_title(&self) -> &str {
        if let Some(title) = &self.title {
            title
        } else {
            &self.sensor_id
        }
    }
}
