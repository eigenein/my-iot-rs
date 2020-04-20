#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    pub sensor_id: String,
    pub title: Option<String>,
    pub room_title: Option<String>,
}

impl Sensor {
    pub fn new<S: Into<String>>(sensor_id: S) -> Self {
        Sensor {
            sensor_id: sensor_id.into(),
            title: None,
            room_title: None,
        }
    }

    pub fn display_title(&self) -> &str {
        if let Some(title) = &self.title {
            title
        } else {
            &self.sensor_id
        }
    }
}
