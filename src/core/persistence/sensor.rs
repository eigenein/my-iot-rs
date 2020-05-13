#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub title: Option<String>,
    pub room_title: Option<String>,
}

impl Sensor {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Sensor {
            id: id.into(),
            title: None,
            room_title: None,
        }
    }
}
