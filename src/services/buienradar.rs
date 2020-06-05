use crate::prelude::*;
use chrono::offset::TimeZone;
use chrono_tz::Europe::Amsterdam;
use reqwest::blocking::Client;
use serde::{de, Deserialize, Deserializer};
use std::thread;
use std::time::Duration;
use uom::si::f64::*;
use uom::si::*;

/// Buienradar JSON feed URL.
const URL: &str = "https://json.buienradar.nl/";
const REFRESH_PERIOD: Duration = Duration::from_millis(60000);

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Buienradar {
    /// Station ID. Find a one [here](https://json.buienradar.nl/).
    station_id: u32,
}

impl Buienradar {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let client = client_builder().build()?;

        thread::Builder::new().name(service_id.clone()).spawn(move || loop {
            if let Err(error) = self.loop_(&client, &service_id, &tx) {
                error!("Failed to refresh the sensors: {}", error.to_string());
            }
            thread::sleep(REFRESH_PERIOD);
        })?;

        Ok(())
    }

    fn loop_(&self, client: &Client, service_id: &str, tx: &Sender) -> Result<()> {
        self.send_readings(self.fetch(&client)?, &service_id, &tx)
    }

    /// Fetch measurement for the configured station.
    fn fetch(&self, client: &Client) -> Result<BuienradarFeedActual> {
        Ok(client.get(URL).send()?.json::<BuienradarFeed>()?.actual)
    }

    /// Sends out readings based on Buienradar station measurement.
    fn send_readings(&self, actual: BuienradarFeedActual, service_id: &str, tx: &Sender) -> Result<()> {
        let measurement = actual
            .station_measurements
            .iter()
            .find(|measurement| measurement.station_id == self.station_id)
            .ok_or_else(|| InternalError::new(format!("station {} is not found", self.station_id)))?;
        tx.send(
            Message::new(format!("{}::{}::weather_description", service_id, self.station_id))
                .type_(MessageType::ReadLogged)
                .value(Value::Text(measurement.weather_description.clone()))
                .timestamp(measurement.timestamp)
                .sensor_title("Description")
                .room_title(&measurement.name),
        )?;
        if let Some(temperature) = measurement.temperature {
            tx.send(
                Message::new(format!("{}::{}::temperature", service_id, self.station_id))
                    .type_(MessageType::ReadLogged)
                    .value(temperature)
                    .timestamp(measurement.timestamp)
                    .sensor_title("Temperature")
                    .room_title(&measurement.name),
            )?;
        }
        if let Some(temperature) = measurement.ground_temperature {
            tx.send(
                Message::new(format!("{}::{}::ground_temperature", service_id, self.station_id))
                    .type_(MessageType::ReadLogged)
                    .value(temperature)
                    .timestamp(measurement.timestamp)
                    .sensor_title("Ground Temperature")
                    .room_title(&measurement.name),
            )?;
        }
        if let Some(temperature) = measurement.feel_temperature {
            tx.send(
                Message::new(format!("{}::{}::feel_temperature", service_id, self.station_id))
                    .type_(MessageType::ReadLogged)
                    .value(temperature)
                    .timestamp(measurement.timestamp)
                    .sensor_title("Feel Temperature")
                    .room_title(&measurement.name),
            )?;
        }
        if let Some(bft) = measurement.wind_speed_bft {
            tx.send(
                Message::new(format!("{}::{}::wind_force", service_id, self.station_id))
                    .type_(MessageType::ReadLogged)
                    .value(Value::Bft(bft))
                    .timestamp(measurement.timestamp)
                    .sensor_title("Wind Force")
                    .room_title(&measurement.name),
            )?;
        }
        if let Some(point) = measurement.wind_direction {
            tx.send(
                Message::new(format!("{}::{}::wind_direction", service_id, self.station_id))
                    .type_(MessageType::ReadLogged)
                    .value(Value::WindDirection(point))
                    .timestamp(measurement.timestamp)
                    .sensor_title("Wind Direction")
                    .room_title(&measurement.name),
            )?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct BuienradarFeed {
    pub actual: BuienradarFeedActual,
}

#[derive(Deserialize)]
struct BuienradarFeedActual {
    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub sunrise: DateTime<Local>,

    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_datetime")]
    pub sunset: DateTime<Local>,

    #[serde(rename = "stationmeasurements")]
    pub station_measurements: Vec<BuienradarStationMeasurement>,
}

#[derive(Deserialize, Clone)]
struct BuienradarStationMeasurement {
    #[serde(rename = "stationid")]
    pub station_id: u32,

    #[serde(rename = "stationname")]
    pub name: String,

    #[serde(default, deserialize_with = "deserialize_temperature")]
    pub temperature: Option<ThermodynamicTemperature>,

    #[serde(default, rename = "groundtemperature", deserialize_with = "deserialize_temperature")]
    pub ground_temperature: Option<ThermodynamicTemperature>,

    #[serde(default, rename = "feeltemperature", deserialize_with = "deserialize_temperature")]
    pub feel_temperature: Option<ThermodynamicTemperature>,

    #[serde(default, rename = "windspeedBft")]
    pub wind_speed_bft: Option<u8>,

    #[serde(deserialize_with = "deserialize_datetime")]
    pub timestamp: DateTime<Local>,

    #[serde(
        default,
        rename = "winddirection",
        deserialize_with = "deserialize_point_of_the_compass"
    )]
    pub wind_direction: Option<PointOfTheCompass>,

    #[serde(rename = "weatherdescription")]
    pub weather_description: String,
}

/// Implements [custom date/time format](https://serde.rs/custom-date-format.html) with Amsterdam timezone.
fn deserialize_datetime<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<DateTime<Local>, D::Error> {
    Ok(Amsterdam
        .datetime_from_str(&String::deserialize(deserializer)?, "%Y-%m-%dT%H:%M:%S")
        .map_err(de::Error::custom)?
        .with_timezone(&Local))
}

/// Translates Dutch wind direction acronyms.
fn deserialize_point_of_the_compass<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<PointOfTheCompass>, D::Error> {
    match Option::<String>::deserialize(deserializer)? {
        None => Ok(None),
        Some(value) => match value.as_str() {
            "N" => Ok(Some(PointOfTheCompass::North)),
            "NNO" => Ok(Some(PointOfTheCompass::NorthNortheast)),
            "NO" => Ok(Some(PointOfTheCompass::Northeast)),
            "ONO" => Ok(Some(PointOfTheCompass::EastNortheast)),
            "O" => Ok(Some(PointOfTheCompass::East)),
            "OZO" => Ok(Some(PointOfTheCompass::EastSoutheast)),
            "ZO" => Ok(Some(PointOfTheCompass::Southeast)),
            "ZZO" => Ok(Some(PointOfTheCompass::SouthSoutheast)),
            "Z" => Ok(Some(PointOfTheCompass::South)),
            "ZZW" => Ok(Some(PointOfTheCompass::SouthSouthwest)),
            "ZW" => Ok(Some(PointOfTheCompass::Southwest)),
            "WZW" => Ok(Some(PointOfTheCompass::WestSouthwest)),
            "W" => Ok(Some(PointOfTheCompass::West)),
            "WNW" => Ok(Some(PointOfTheCompass::WestNorthwest)),
            "NW" => Ok(Some(PointOfTheCompass::Northwest)),
            "NNW" => Ok(Some(PointOfTheCompass::NorthNorthwest)),
            value => Err(de::Error::custom(format!(
                "could not translate wind direction: {}",
                value
            ))),
        },
    }
}

fn deserialize_temperature<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<ThermodynamicTemperature>, D::Error> {
    Ok(Option::<f64>::deserialize(deserializer)?
        .map(ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>))
}

#[cfg(test)]
mod tests {
    use crate::services::buienradar::BuienradarFeed;
    use crate::Result;
    use uom::si::f64::*;
    use uom::si::*;

    #[test]
    fn parse() -> Result<()> {
        let feed = serde_json::from_str::<BuienradarFeed>(
            r#"{"$id":"1","buienradar":{"$id":"2","copyright":"(C)opyright Buienradar / RTL. Alle rechten voorbehouden","terms":"Deze feed mag vrij worden gebruikt onder voorwaarde van bronvermelding buienradar.nl inclusief een hyperlink naar https://www.buienradar.nl. Aan de feed kunnen door gebruikers of andere personen geen rechten worden ontleend."},"actual":{"$id":"3","actualradarurl":"https://api.buienradar.nl/image/1.0/RadarMapNL?w=500&h=512","sunrise":"2019-08-24T06:37:00","sunset":"2019-08-24T20:45:00","stationmeasurements":[{"$id":"4","stationid":6391,"stationname":"Meetstation Arcen","lat":51.5,"lon":6.2,"regio":"Venlo","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"N","temperature":24.2,"groundtemperature":22.4,"feeltemperature":25.7,"windgusts":2.1,"windspeed":1.3,"windspeedBft":1,"humidity":62.0,"precipitation":0.0,"sunpower":20.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":11},{"$id":"5","stationid":6275,"stationname":"Meetstation Arnhem","lat":52.07,"lon":5.88,"regio":"Arnhem","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.8,"temperature":24.7,"groundtemperature":23.7,"feeltemperature":26.0,"visibility":34900.0,"windgusts":1.6,"windspeed":1.3,"windspeedBft":1,"humidity":52.0,"precipitation":0.0,"sunpower":56.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":69},{"$id":"6","stationid":6249,"stationname":"Meetstation Berkhout","lat":52.65,"lon":4.98,"regio":"Berkhout","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"NNO","temperature":25.4,"groundtemperature":25.3,"feeltemperature":26.6,"visibility":42500.0,"windgusts":1.4,"windspeed":0.6,"windspeedBft":1,"humidity":59.0,"precipitation":0.0,"sunpower":64.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":30},{"$id":"7","stationid":6308,"stationname":"Meetstation Cadzand","lat":51.38,"lon":3.38,"regio":"Cadzand","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"ONO","windgusts":3.6,"windspeed":2.6,"windspeedBft":2,"winddirectiondegrees":77},{"$id":"8","stationid":6260,"stationname":"Meetstation De Bilt","lat":52.1,"lon":5.18,"regio":"Utrecht","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ZO","airpressure":1018.7,"temperature":26.6,"groundtemperature":24.7,"feeltemperature":27.5,"visibility":37000.0,"windgusts":1.9,"windspeed":1.2,"windspeedBft":1,"humidity":53.0,"precipitation":0.0,"sunpower":60.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":127},{"$id":"9","stationid":6235,"stationname":"Meetstation Den Helder","lat":52.92,"lon":4.78,"regio":"Den Helder","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"NNO","airpressure":1019.1,"temperature":22.7,"groundtemperature":22.2,"feeltemperature":24.0,"visibility":34300.0,"windgusts":3.9,"windspeed":2.5,"windspeedBft":2,"humidity":73.0,"precipitation":0.0,"sunpower":65.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":33},{"$id":"10","stationid":6370,"stationname":"Meetstation Eindhoven","lat":51.45,"lon":5.42,"regio":"Eindhoven","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.2,"temperature":28.3,"groundtemperature":26.7,"feeltemperature":27.7,"visibility":43400.0,"windgusts":4.5,"windspeed":2.4,"windspeedBft":2,"humidity":34.0,"precipitation":0.0,"sunpower":69.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":69},{"$id":"11","stationid":6377,"stationname":"Meetstation Ell","lat":51.2,"lon":5.77,"regio":"Weert","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"NNO","temperature":26.4,"groundtemperature":22.7,"feeltemperature":26.6,"visibility":32600.0,"windgusts":1.8,"windspeed":1.2,"windspeedBft":1,"humidity":40.0,"precipitation":0.0,"sunpower":47.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":29},{"$id":"12","stationid":6321,"stationname":"Meetstation Euro platform","lat":52.0,"lon":3.28,"regio":"Noordzee","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","airpressure":1018.5,"visibility":10900.0,"windgusts":7.2,"windspeed":5.9,"windspeedBft":4,"winddirectiondegrees":45},{"$id":"13","stationid":6350,"stationname":"Meetstation Gilze Rijen","lat":51.57,"lon":4.93,"regio":"Gilze Rijen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.2,"temperature":26.5,"groundtemperature":25.0,"feeltemperature":26.9,"visibility":34000.0,"windgusts":4.4,"windspeed":2.7,"windspeedBft":2,"humidity":43.0,"precipitation":0.0,"sunpower":59.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":70},{"$id":"14","stationid":6323,"stationname":"Meetstation Goes","lat":51.53,"lon":3.9,"regio":"Goes","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"NNO","airpressure":1018.2,"temperature":26.3,"groundtemperature":24.2,"feeltemperature":26.9,"windgusts":3.4,"windspeed":2.0,"windspeedBft":2,"humidity":46.0,"precipitation":0.0,"sunpower":70.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":19},{"$id":"15","stationid":6283,"stationname":"Meetstation Groenlo-Hupsel","lat":52.07,"lon":6.65,"regio":"Oost-Overijssel","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","temperature":24.9,"groundtemperature":22.4,"feeltemperature":26.0,"windgusts":2.2,"windspeed":1.4,"windspeedBft":1,"humidity":48.0,"precipitation":0.0,"sunpower":53.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":61},{"$id":"16","stationid":6280,"stationname":"Meetstation Groningen","lat":53.13,"lon":6.58,"regio":"Groningen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"O","airpressure":1019.9,"temperature":25.5,"groundtemperature":23.2,"feeltemperature":26.2,"visibility":42100.0,"windgusts":2.6,"windspeed":1.8,"windspeedBft":2,"humidity":41.0,"precipitation":0.0,"sunpower":48.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":95},{"$id":"17","stationid":6315,"stationname":"Meetstation Hansweert","lat":51.45,"lon":4.0,"regio":"Oost-Zeeland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":1.9,"windspeed":1.3,"windspeedBft":1,"winddirectiondegrees":46},{"$id":"18","stationid":6278,"stationname":"Meetstation Heino","lat":52.43,"lon":6.27,"regio":"Zwolle","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","temperature":23.8,"groundtemperature":23.2,"feeltemperature":25.5,"windgusts":2.4,"windspeed":0.8,"windspeedBft":1,"humidity":57.0,"precipitation":0.0,"sunpower":54.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":67},{"$id":"19","stationid":6356,"stationname":"Meetstation Herwijnen","lat":51.87,"lon":5.15,"regio":"Gorinchem","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"NO","airpressure":1018.5,"temperature":25.1,"groundtemperature":24.0,"feeltemperature":26.3,"windgusts":2.6,"windspeed":1.8,"windspeedBft":2,"humidity":56.0,"precipitation":0.0,"sunpower":58.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":43},{"$id":"20","stationid":6330,"stationname":"Meetstation Hoek van Holland","lat":51.98,"lon":4.1,"regio":"Hoek van Holland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","airpressure":1018.5,"temperature":23.7,"groundtemperature":23.0,"feeltemperature":25.1,"windgusts":4.4,"windspeed":2.4,"windspeedBft":2,"humidity":68.0,"precipitation":0.0,"sunpower":34.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":35},{"$id":"21","stationid":6279,"stationname":"Meetstation Hoogeveen","lat":52.73,"lon":6.52,"regio":"Hoogeveen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1019.5,"temperature":24.4,"groundtemperature":23.4,"feeltemperature":25.8,"visibility":43800.0,"windgusts":3.4,"windspeed":2.6,"windspeedBft":2,"humidity":51.0,"precipitation":0.0,"sunpower":55.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":69},{"$id":"22","stationid":6251,"stationname":"Meetstation Hoorn Terschelling","lat":53.38,"lon":5.35,"regio":"Wadden","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1019.7,"temperature":21.5,"groundtemperature":20.8,"feeltemperature":21.5,"visibility":29300.0,"windgusts":5.4,"windspeed":4.2,"windspeedBft":3,"humidity":73.0,"precipitation":0.0,"sunpower":68.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":66},{"$id":"23","stationid":6258,"stationname":"Meetstation Houtribdijk","lat":52.65,"lon":5.4,"regio":"Enkhuizen-Lelystad","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"O","windgusts":7.0,"windspeed":5.6,"windspeedBft":4,"winddirectiondegrees":87},{"$id":"24","stationid":6285,"stationname":"Meetstation Huibertgat","lat":53.57,"lon":6.4,"regio":"Schiermonnikoog","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"ONO","windgusts":8.8,"windspeed":7.7,"windspeedBft":4,"winddirectiondegrees":70},{"$id":"25","stationid":6209,"stationname":"Meetstation IJmond","lat":52.47,"lon":4.52,"regio":"IJmond","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NW","windgusts":0.7,"windspeed":0.3,"windspeedBft":0,"winddirectiondegrees":321},{"$id":"26","stationid":6225,"stationname":"Meetstation IJmuiden","lat":52.47,"lon":4.57,"regio":"IJmuiden","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"WNW","windgusts":1.0,"windspeed":0.6,"windspeedBft":1,"winddirectiondegrees":289},{"$id":"27","stationid":6277,"stationname":"Meetstation Lauwersoog","lat":53.42,"lon":6.2,"regio":"Noord-Groningen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"O","temperature":23.2,"groundtemperature":22.4,"feeltemperature":24.6,"windgusts":5.9,"windspeed":5.3,"windspeedBft":3,"humidity":70.0,"precipitation":0.0,"sunpower":48.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":86},{"$id":"28","stationid":6320,"stationname":"Meetstation LE Goeree","lat":51.93,"lon":3.67,"regio":"Goeree","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","airpressure":1018.6,"visibility":12900.0,"windgusts":7.9,"windspeed":6.3,"windspeedBft":4,"winddirectiondegrees":56},{"$id":"29","stationid":6270,"stationname":"Meetstation Leeuwarden","lat":53.22,"lon":5.77,"regio":"Leeuwarden","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"O","airpressure":1019.6,"temperature":24.4,"groundtemperature":21.8,"feeltemperature":25.8,"visibility":29000.0,"windgusts":3.9,"windspeed":3.0,"windspeedBft":2,"humidity":51.0,"precipitation":0.0,"sunpower":48.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":89},{"$id":"30","stationid":6269,"stationname":"Meetstation Lelystad","lat":52.45,"lon":5.53,"regio":"Lelystad","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt en regen","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/q.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/q","winddirection":"O","airpressure":1018.9,"temperature":26.0,"groundtemperature":23.2,"feeltemperature":26.4,"visibility":47100.0,"windgusts":3.2,"windspeed":2.4,"windspeedBft":2,"humidity":40.0,"precipitation":0.0,"sunpower":69.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":95},{"$id":"31","stationid":6348,"stationname":"Meetstation Lopik-Cabauw","lat":51.97,"lon":4.93,"regio":"West-Utrecht","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.6,"temperature":26.0,"groundtemperature":22.6,"feeltemperature":26.7,"visibility":25800.0,"windgusts":2.5,"windspeed":1.5,"windspeedBft":1,"humidity":47.0,"precipitation":0.0,"sunpower":64.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":69},{"$id":"32","stationid":6380,"stationname":"Meetstation Maastricht","lat":50.92,"lon":5.78,"regio":"Maastricht","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.0,"temperature":25.6,"groundtemperature":22.5,"feeltemperature":26.2,"visibility":48400.0,"windgusts":4.3,"windspeed":2.8,"windspeedBft":2,"humidity":40.0,"precipitation":0.0,"sunpower":58.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":71},{"$id":"33","stationid":6273,"stationname":"Meetstation Marknesse","lat":52.7,"lon":5.88,"regio":"Noordoostpolder","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","temperature":24.8,"groundtemperature":23.8,"feeltemperature":26.1,"visibility":37700.0,"windgusts":2.8,"windspeed":2.1,"windspeedBft":2,"humidity":56.0,"precipitation":0.0,"sunpower":57.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":68},{"$id":"34","stationid":6286,"stationname":"Meetstation Nieuw Beerta","lat":53.2,"lon":7.15,"regio":"Oost-Groningen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","temperature":25.0,"groundtemperature":23.9,"feeltemperature":26.1,"windgusts":4.1,"windspeed":2.9,"windspeedBft":2,"humidity":50.0,"precipitation":0.0,"sunpower":50.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":66},{"$id":"35","stationid":6312,"stationname":"Meetstation Oosterschelde","lat":51.77,"lon":3.62,"regio":"Oosterschelde","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":6.6,"windspeed":5.9,"windspeedBft":4,"winddirectiondegrees":40},{"$id":"36","stationid":6344,"stationname":"Meetstation Rotterdam","lat":51.95,"lon":4.45,"regio":"Rotterdam","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","airpressure":1018.5,"temperature":25.3,"groundtemperature":22.6,"feeltemperature":26.3,"visibility":34700.0,"windgusts":3.2,"windspeed":2.3,"windspeedBft":2,"humidity":49.0,"precipitation":0.0,"sunpower":74.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":56},{"$id":"37","stationid":6343,"stationname":"Meetstation Rotterdam Geulhaven","lat":51.88,"lon":4.32,"regio":"Rotterdam Haven","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"O","windgusts":5.8,"windspeed":3.5,"windspeedBft":3,"winddirectiondegrees":85},{"$id":"38","stationid":6316,"stationname":"Meetstation Schaar","lat":51.65,"lon":3.7,"regio":"Schaar","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NNO","windgusts":4.9,"windspeed":4.1,"windspeedBft":3,"winddirectiondegrees":31},{"$id":"39","stationid":6240,"stationname":"Meetstation Schiphol","lat":52.3,"lon":4.77,"regio":"Amsterdam","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"O","airpressure":1018.7,"temperature":26.6,"groundtemperature":21.6,"feeltemperature":26.7,"visibility":47200.0,"windgusts":2.0,"windspeed":1.1,"windspeedBft":1,"humidity":39.0,"precipitation":0.0,"sunpower":71.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":84},{"$id":"40","stationid":6324,"stationname":"Meetstation Stavenisse","lat":51.6,"lon":4.0,"regio":"Midden-Zeeland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":3.7,"windspeed":2.7,"windspeedBft":2,"winddirectiondegrees":42},{"$id":"41","stationid":6267,"stationname":"Meetstation Stavoren","lat":52.88,"lon":5.38,"regio":"West-Friesland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"O","temperature":24.6,"groundtemperature":22.6,"feeltemperature":25.9,"visibility":37100.0,"windgusts":3.2,"windspeed":2.4,"windspeedBft":2,"humidity":53.0,"precipitation":0.0,"sunpower":64.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":79},{"$id":"42","stationid":6229,"stationname":"Meetstation Texelhors","lat":53.0,"lon":4.75,"regio":"Texel","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":4.2,"windspeed":3.0,"windspeedBft":2,"winddirectiondegrees":51},{"$id":"43","stationid":6331,"stationname":"Meetstation Tholen","lat":51.52,"lon":4.13,"regio":"Tholen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":1.0,"windspeed":0.6,"windspeedBft":1,"winddirectiondegrees":50},{"$id":"44","stationid":6290,"stationname":"Meetstation Twente","lat":52.27,"lon":6.9,"regio":"Twente","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1019.0,"temperature":26.5,"groundtemperature":23.9,"feeltemperature":26.5,"visibility":49600.0,"windgusts":3.0,"windspeed":2.1,"windspeedBft":2,"humidity":36.0,"precipitation":0.0,"sunpower":51.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":73},{"$id":"45","stationid":6313,"stationname":"Meetstation Vlakte aan de Raan","lat":51.5,"lon":3.25,"regio":"West-Zeeland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"NO","windgusts":6.5,"windspeed":5.3,"windspeedBft":3,"winddirectiondegrees":44},{"$id":"46","stationid":6242,"stationname":"Meetstation Vlieland","lat":53.25,"lon":4.92,"regio":"Vlieland","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"ONO","airpressure":1019.4,"temperature":23.0,"groundtemperature":22.4,"feeltemperature":24.0,"visibility":32900.0,"windgusts":6.0,"windspeed":4.6,"windspeedBft":3,"humidity":75.0,"winddirectiondegrees":67},{"$id":"47","stationid":6310,"stationname":"Meetstation Vlissingen","lat":51.45,"lon":3.6,"regio":"Vlissingen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"O","airpressure":1018.0,"temperature":26.7,"groundtemperature":24.4,"feeltemperature":26.8,"visibility":42300.0,"windgusts":3.2,"windspeed":2.4,"windspeedBft":2,"humidity":39.0,"precipitation":0.0,"sunpower":68.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":91},{"$id":"48","stationid":6375,"stationname":"Meetstation Volkel","lat":51.65,"lon":5.7,"regio":"Uden","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"ONO","airpressure":1018.3,"temperature":25.9,"groundtemperature":23.2,"feeltemperature":26.4,"visibility":37500.0,"windgusts":2.8,"windspeed":2.0,"windspeedBft":2,"humidity":41.0,"precipitation":0.0,"sunpower":57.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":58},{"$id":"49","stationid":6215,"stationname":"Meetstation Voorschoten","lat":52.12,"lon":4.43,"regio":"Voorschoten","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"O","airpressure":1018.6,"temperature":25.3,"groundtemperature":23.3,"feeltemperature":26.3,"visibility":33600.0,"windgusts":2.5,"windspeed":1.8,"windspeedBft":2,"humidity":51.0,"precipitation":0.0,"sunpower":68.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":79},{"$id":"50","stationid":6319,"stationname":"Meetstation Westdorpe","lat":51.23,"lon":3.83,"regio":"Terneuzen","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"OZO","airpressure":1018.1,"temperature":27.2,"groundtemperature":25.6,"feeltemperature":27.5,"visibility":43500.0,"windgusts":1.9,"windspeed":0.9,"windspeedBft":1,"humidity":44.0,"precipitation":0.0,"sunpower":70.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":102},{"$id":"51","stationid":6248,"stationname":"Meetstation Wijdenes","lat":52.63,"lon":5.17,"regio":"Hoorn","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"ONO","windgusts":3.9,"windspeed":3.4,"windspeedBft":2,"winddirectiondegrees":78},{"$id":"52","stationid":6257,"stationname":"Meetstation Wijk aan Zee","lat":52.5,"lon":4.6,"regio":"Wijk aan Zee","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","temperature":25.4,"groundtemperature":26.2,"humidity":50.0,"precipitation":0.0,"sunpower":69.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0},{"$id":"53","stationid":6340,"stationname":"Meetstation Woensdrecht","lat":51.45,"lon":4.33,"regio":"Woensdrecht","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"O","airpressure":1017.6,"temperature":26.2,"groundtemperature":26.0,"feeltemperature":27.0,"visibility":47900.0,"windgusts":1.6,"windspeed":1.0,"windspeedBft":1,"humidity":50.0,"precipitation":0.0,"rainFallLast24Hour":0.0,"rainFallLastHour":0.0,"winddirectiondegrees":81},{"$id":"54","stationid":6239,"stationname":"Meetstation Zeeplatform F-3","lat":54.85,"lon":4.73,"regio":"Noordzee","timestamp":"2019-08-24T20:00:00","weatherdescription":"Vrijwel onbewolkt (zonnig/helder)","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/a.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/a","winddirection":"OZO","airpressure":1020.6,"temperature":19.6,"feeltemperature":19.6,"visibility":18000.0,"windgusts":7.5,"windspeed":6.8,"windspeedBft":4,"humidity":85.0,"winddirectiondegrees":115},{"$id":"55","stationid":6252,"stationname":"Meetstation Zeeplatform K13","lat":53.22,"lon":3.22,"regio":"Noordzee","timestamp":"2019-08-24T20:00:00","weatherdescription":"Zwaar bewolkt","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/c.png","graphUrl":"https://www.buienradar.nl/nederland/weerbericht/weergrafieken/c","winddirection":"OZO","airpressure":1023.6,"windgusts":7.9,"windspeed":6.2,"windspeedBft":4,"winddirectiondegrees":120}]},"forecast":{"$id":"56","weatherreport":{"$id":"57","published":"2019-08-24T17:45:00","title":"Zonnig en zeer warm","summary":"De zomer is dit weekend en begin volgende week weer helemaal terug! De zon schijnt en het wordt vanaf zondag op  meerdere plaatsen tropisch warm.","text":"De zomer is dit weekend en begin volgende week weer helemaal terug! De zon schijnt en het wordt vanaf zondag op&nbsp; meerdere plaatsen tropisch warm.Vanavond blijft het helder en nog lang warm. De nacht verloopt helder en uiteindelijk koelt het af naar 13 graden in het binnenland en 18 graden aan zee. Er waait een zwakke noordoostenwind.Zondag&nbsp;schijnt de zon opnieuw volop en wordt het nog wat warmer. In het westen en midden van het land kan wat sluierbewolking overtrekken. De temperatuur varieert van 26 graden op de Wadden, 29 graden in het midden van Nederland en 31 graden in het oosten. Er waait een zwakke oostenwind die in de loop van de middag draait naar het noorden. In de middag staat langs de westkust een zwakke tot matige noordwestenwind die opnieuw voor wat verkoeling kan zorgen.&nbsp;De dagen erna gaat de zomer onverminderd door! De zon schijnt volop en slechts af en toe&nbsp;trekken wat wolken over. Het blijft tot en met woensdag&nbsp;vrijwel droog. In de middag wordt het 26 tot 32 graden. Het is niet uitgesloten dat het zelfs nog tot een landelijk hittegolf komt. Later in het seizoen dan 26 augustus is het nog nooit tot een hittegolf gekomen.&nbsp;Dit jaar zou de hittegolf zelfs kunnen duren tot 28 augustus. Naast de hoge temperaturen overdag verlopen de nachten ook zwoel.&nbsp; Op donderdag&nbsp;volgt verkoeling door enkele onweersbuien.","author":"Maurice Middendorp","authorbio":"Sinds 2013 actief als meteoroloog bij Buienradar. Vanaf 2015 ook als presentator van weerberichten op tv en internet. Daarnaast ex-luchtvaartmeteoroloog bij defensie."},"shortterm":{"$id":"58","startdate":"2019-08-25T00:00:00","enddate":"2019-08-29T00:00:00","forecast":"Zonnige perioden en tot en met maandag droog. Vanaf dinsdag een kleine kans op een enkele bui, maar vooral op donderdag kans op regen. Middagtemperatuur tot en met woensdag rond 29 graden met op veel plaatsen kans op tropische waarden. Vanaf donderdag beduidend minder warm."},"longterm":{"$id":"59","startdate":"2019-08-30T00:00:00","enddate":"2019-09-03T00:00:00","forecast":"Kans (40%) op het aanhouden van het licht wisselvallig weertype met circa 60% kans op middagtemperaturen rond de langjaargemiddelde waarden."},"fivedayforecast":[{"$id":"60","day":"2019-08-25T00:00:00","mintemperature":"13","maxtemperature":"30","mintemperatureMax":13,"mintemperatureMin":13,"maxtemperatureMax":30,"maxtemperatureMin":30,"rainChance":10,"sunChance":90,"windDirection":"var","wind":2,"mmRainMin":0.0,"mmRainMax":0.0,"weatherdescription":"Mix van opklaringen en middelbare of lage bewolking","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/b.png"},{"$id":"61","day":"2019-08-26T00:00:00","mintemperature":"15/16","maxtemperature":"28/29","mintemperatureMax":16,"mintemperatureMin":15,"maxtemperatureMax":29,"maxtemperatureMin":28,"rainChance":10,"sunChance":70,"windDirection":"o","wind":2,"mmRainMin":0.0,"mmRainMax":0.0,"weatherdescription":"Mix van opklaringen en middelbare of lage bewolking","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/b.png"},{"$id":"62","day":"2019-08-27T00:00:00","mintemperature":"17/18","maxtemperature":"28/29","mintemperatureMax":18,"mintemperatureMin":17,"maxtemperatureMax":29,"maxtemperatureMin":28,"rainChance":30,"sunChance":50,"windDirection":"o","wind":2,"mmRainMin":0.0,"mmRainMax":1.0,"weatherdescription":"Mix van opklaringen en middelbare of lage bewolking","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/b.png"},{"$id":"63","day":"2019-08-28T00:00:00","mintemperature":"17/19","maxtemperature":"27/30","mintemperatureMax":19,"mintemperatureMin":17,"maxtemperatureMax":30,"maxtemperatureMin":27,"rainChance":20,"sunChance":30,"windDirection":"z","wind":2,"mmRainMin":0.0,"mmRainMax":1.0,"weatherdescription":"Mix van opklaringen en middelbare of lage bewolking","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/b.png"},{"$id":"64","day":"2019-08-29T00:00:00","mintemperature":"17/19","maxtemperature":"22/25","mintemperatureMax":19,"mintemperatureMin":17,"maxtemperatureMax":25,"maxtemperatureMin":22,"rainChance":40,"sunChance":40,"windDirection":"w","wind":3,"mmRainMin":0.0,"mmRainMax":4.0,"weatherdescription":"Afwisselend bewolkt met (mogelijk) wat lichte regen","iconurl":"https://www.buienradar.nl/resources/images/icons/weather/30x30/f.png"}]}}"#,
        )?;
        assert_eq!(
            feed.actual.station_measurements[0].temperature,
            Some(ThermodynamicTemperature::new::<thermodynamic_temperature::degree_celsius>(24.2)),
            "{:?}",
            feed.actual.station_measurements[0].temperature,
        );
        Ok(())
    }
}
