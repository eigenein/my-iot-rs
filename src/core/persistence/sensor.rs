#[derive(PartialEq, Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub title: Option<String>,
    pub room_title: Option<String>,
}
