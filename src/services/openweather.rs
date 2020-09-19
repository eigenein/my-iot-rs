use surf::url::Url;

use crate::prelude::*;
use crate::services::prelude::*;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct OpenWeather {
    secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct Secrets {
    api_key: String,
    latitude: f64,
    longitude: f64,
}

/// <https://openweathermap.org/current>
impl OpenWeather {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let mut tx = bus.add_tx();
        task::spawn(async move {
            loop {
                handle_service_result(&service_id, MINUTE, self.loop_(&service_id, &mut tx).await).await;
            }
        });
        Ok(())
    }

    async fn loop_(&self, service_id: &str, tx: &mut Sender) -> Result {
        let response = CLIENT
            .get(Url::parse_with_params(
                "https://api.openweathermap.org/data/2.5/weather",
                &[
                    ("units", "metric"),
                    ("lang", "en"),
                    ("appid", &self.secrets.api_key),
                    ("lat", &self.secrets.latitude.to_string()),
                    ("lon", &self.secrets.longitude.to_string()),
                ],
            )?)
            .recv_json::<Response>()
            .await
            .map_err(|err| anyhow!(err))?;

        let sensor_prefix = format!("{}::{}", service_id, response.city_id);

        Message::new(format!("{}::temperature", sensor_prefix))
            .value(Value::Temperature(response.main.temperature))
            .timestamp(response.timestamp)
            .sensor_title("Temperature")
            .location(&response.city_name)
            .send_to(tx)
            .await;
        Message::new(format!("{}::temperature::feel", sensor_prefix))
            .value(Value::Temperature(response.main.feel_temperature))
            .timestamp(response.timestamp)
            .sensor_title("Feel Temperature")
            .location(&response.city_name)
            .send_to(tx)
            .await;
        Message::new(format!("{}::temperature::min", sensor_prefix))
            .value(Value::Temperature(response.main.temperature_min))
            .timestamp(response.timestamp)
            .sensor_title("Minimal Temperature")
            .location(&response.city_name)
            .send_to(tx)
            .await;
        Message::new(format!("{}::temperature::max", sensor_prefix))
            .value(Value::Temperature(response.main.temperature_max))
            .timestamp(response.timestamp)
            .sensor_title("Maximal Temperature")
            .location(&response.city_name)
            .send_to(tx)
            .await;

        Message::new(format!("{}::wind::speed", sensor_prefix))
            .value(Value::Speed(response.wind.speed))
            .timestamp(response.timestamp)
            .sensor_title("Wind Speed")
            .location(&response.city_name)
            .send_to(tx)
            .await;
        if let Some(speed) = response.wind.gusts {
            Message::new(format!("{}::wind::gusts", sensor_prefix))
                .value(Value::Speed(speed))
                .timestamp(response.timestamp)
                .sensor_title("Wind Gusts")
                .location(&response.city_name)
                .send_to(tx)
                .await;
        }

        Message::new(format!("{}::cloudiness", sensor_prefix))
            .value(Value::Cloudiness(response.clouds.all))
            .timestamp(response.timestamp)
            .sensor_title("Cloudiness")
            .location(&response.city_name)
            .send_to(tx)
            .await;

        Message::new(format!("{}::rain::last_hour", sensor_prefix))
            .value(Value::from_mm(response.rain.last_hour))
            .timestamp(response.timestamp)
            .sensor_title("Rain Last Hour")
            .location(&response.city_name)
            .send_to(tx)
            .await;

        Ok(())
    }
}

#[derive(Deserialize)]
struct Response {
    #[serde(rename = "dt", deserialize_with = "deserialize_timestamp")]
    timestamp: DateTime<Local>,

    #[serde(rename = "name")]
    city_name: String,

    #[serde(rename = "id")]
    city_id: u32,

    main: ResponseMain,

    wind: ResponseWind,

    #[serde(default)]
    rain: ResponseRain,

    clouds: ResponseClouds,
}

#[derive(Deserialize)]
struct ResponseMain {
    #[serde(rename = "temp")]
    temperature: f64,

    /// Minimum temperature at the moment.
    /// This is minimal currently observed temperature (within large megalopolises and urban areas).
    #[serde(rename = "temp_min")]
    temperature_min: f64,

    /// Maximum temperature at the moment.
    /// This is maximal currently observed temperature (within large megalopolises and urban areas).
    #[serde(rename = "temp_max")]
    temperature_max: f64,

    /// Temperature. This temperature parameter accounts for the human perception of weather.
    #[serde(rename = "feels_like")]
    feel_temperature: f64,
}

#[derive(Deserialize)]
struct ResponseWind {
    speed: f64,

    #[serde(rename = "gust")]
    gusts: Option<f64>,
}

#[derive(Deserialize)]
struct ResponseRain {
    /// Rain volume for the last 1 hour, mm.
    #[serde(rename = "1h")]
    last_hour: f64,
}

impl Default for ResponseRain {
    fn default() -> Self {
        Self { last_hour: 0.0 }
    }
}

#[derive(Deserialize)]
struct ResponseClouds {
    /// Cloudiness, %.
    all: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() -> Result {
        serde_json::from_str::<Response>(
            r#"{"coord":{"lon":4.66,"lat":52.36},"weather":[{"id":801,"main":"Clouds","description":"few clouds","icon":"02d"}],"base":"stations","main":{"temp":19.47,"feels_like":14.2,"temp_min":18.33,"temp_max":20,"pressure":1010,"humidity":60},"visibility":10000,"wind":{"speed":8.2,"deg":260},"clouds":{"all":20},"dt":1593698008,"sys":{"type":1,"id":1524,"country":"NL","sunrise":1593660273,"sunset":1593720364},"timezone":7200,"id":2747702,"name":"Schalkwijk","cod":200}"#,
        )?;
        serde_json::from_str::<Response>(
            r#"{"coord":{"lon":27.57,"lat":53.9},"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04d"}],"base":"stations","main":{"temp":25,"feels_like":24.97,"temp_min":25,"temp_max":25,"pressure":1009,"humidity":65},"visibility":10000,"wind":{"speed":4,"deg":250},"clouds":{"all":75},"dt":1593699247,"sys":{"type":1,"id":8939,"country":"BY","sunrise":1593654214,"sunset":1593715424},"timezone":10800,"id":625143,"name":"Minsk City","cod":200}"#,
        )?;
        serde_json::from_str::<Response>(
            r#"{"coord":{"lon":37.62,"lat":55.76},"weather":[{"id":503,"main":"Rain","description":"very heavy rain","icon":"10d"}],"base":"stations","main":{"temp":22.38,"feels_like":20.92,"temp_min":20.56,"temp_max":25,"pressure":1011,"humidity":60},"visibility":10000,"wind":{"speed":4,"deg":190},"rain":{"1h":44.96},"clouds":{"all":40},"dt":1593699165,"sys":{"type":1,"id":9027,"country":"RU","sunrise":1593651041,"sunset":1593713773},"timezone":10800,"id":524925,"name":"Moscow Oblast","cod":200}"#,
        )?;
        serde_json::from_str::<Response>(
            r#"{"coord":{"lon":-79.37,"lat":43.74},"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"base":"stations","main":{"temp":28.68,"feels_like":26.54,"temp_min":28,"temp_max":29,"pressure":1014,"humidity":42},"visibility":14484,"wind":{"speed":5.1,"deg":320,"gust":8.2},"clouds":{"all":1},"dt":1593699491,"sys":{"type":1,"id":941,"country":"CA","sunrise":1593682811,"sunset":1593738165},"timezone":-14400,"id":5941602,"name":"Don Mills","cod":200}"#,
        )?;
        serde_json::from_str::<Response>(
            r#"{"coord":{"lon":151.21,"lat":-33.87},"weather":[{"id":801,"main":"Clouds","description":"few clouds","icon":"02n"}],"base":"stations","main":{"temp":15.26,"feels_like":13.49,"temp_min":13.89,"temp_max":16.67,"pressure":1017,"humidity":77},"visibility":10000,"wind":{"speed":3.1,"deg":240},"clouds":{"all":22},"dt":1593699589,"sys":{"type":1,"id":9600,"country":"AU","sunrise":1593723646,"sunset":1593759459},"timezone":36000,"id":6619279,"name":"Sydney","cod":200}"#,
        )?;
        Ok(())
    }
}
