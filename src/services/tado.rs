//! [tado°](https://www.tado.com/) API.

use crate::prelude::*;
use crate::services::prelude::*;
use reqwest::Url;
use std::thread;
use std::time::{Duration, SystemTime};

const CLIENT_ID: &str = "public-api-preview";
const CLIENT_SECRET: &str = "4HJGRffVR8xb3XdEUQpjgZ1VplJi6Xgw";
const SCOPE: &str = "home.user";
const REFRESH_PERIOD: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Tado {
    secrets: Secrets,

    #[serde(skip, default = "default_client")]
    client: Client,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    email: String,
    password: String,
}

impl Tado {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut token: Option<Token> = None;
            loop {
                if let Err(error) = self.loop_(&service_id, &mut token, &tx) {
                    error!("Failed to refresh the sensors: {}", error.to_string());
                }
                thread::sleep(REFRESH_PERIOD);
            }
        })?;

        Ok(())
    }

    fn loop_(&self, service_id: &str, token: &mut Option<Token>, tx: &Sender) -> Result<()> {
        let access_token = self.check_login(token)?;
        let ttl = chrono::Duration::seconds(120);

        let me = self.get_me(&access_token)?;
        let home = self.get_home(&access_token, me.home_id)?;
        let weather = self.get_weather(&access_token, me.home_id)?;

        Message::new(format!("{}::{}::solar_intensity", service_id, me.home_id))
            .timestamp(weather.solar_intensity.timestamp)
            .expires_in(ttl)
            .value(Value::RelativeIntensity(weather.solar_intensity.percentage))
            .room_title(&home.name)
            .sensor_title("Solar Intensity")
            .send_and_forget(tx);

        let home_state = self.get_home_state(&access_token, me.home_id)?;

        Message::new(format!("{}::{}::is_home", service_id, me.home_id))
            .expires_in(ttl)
            .value(home_state.presence == Presence::Home)
            .room_title(&home.name)
            .sensor_title("Is Home")
            .send_and_forget(tx);

        for zone in self.get_zones(&access_token, me.home_id)?.iter() {
            let sensor_prefix = format!("{}::{}::{}", service_id, me.home_id, zone.id);
            let zone_state = self.get_zone_state(&access_token, me.home_id, zone.id)?;

            Message::new(format!("{}::is_online", sensor_prefix))
                .value(zone_state.link.state == LinkState::Online)
                .expires_in(ttl)
                .room_title(&zone.name)
                .sensor_title("Is Online")
                .send_and_forget(tx);
            Message::new(format!("{}::is_on", sensor_prefix))
                .expires_in(ttl)
                .value(zone_state.setting.power == PowerState::On)
                .room_title(&zone.name)
                .sensor_title(format!(
                    "Is {} On",
                    match zone.type_ {
                        ZoneType::Heating => "Heating",
                        ZoneType::HotWater => "Hot Water",
                    }
                ))
                .send_and_forget(tx);

            if let ZoneSettingAttributes::Heating { temperature } = zone_state.setting.attributes {
                Message::new(format!("{}::set_temperature", sensor_prefix))
                    .expires_in(ttl)
                    .value(Value::Temperature(temperature.celsius))
                    .room_title(&zone.name)
                    .sensor_title("Set Temperature")
                    .send_and_forget(tx);
            }

            if let Some(humidity) = zone_state.sensor_data_points.humidity {
                Message::new(format!("{}::humidity", sensor_prefix))
                    .timestamp(humidity.timestamp)
                    .expires_in(ttl)
                    .room_title(&zone.name)
                    .sensor_title("Humidity")
                    .value(Value::Rh(humidity.percentage))
                    .send_and_forget(tx);
            }

            if let Some(temperature) = zone_state.sensor_data_points.inside_temperature {
                Message::new(format!("{}::temperature", sensor_prefix))
                    .timestamp(temperature.timestamp)
                    .expires_in(ttl)
                    .room_title(&zone.name)
                    .sensor_title("Temperature")
                    .value(Value::Temperature(temperature.celsius))
                    .send_and_forget(tx);
            }
        }

        Ok(())
    }

    /// Checks if the service is logged in. Logs in or refreshes access token when needed.
    /// Returns an active access token.
    fn check_login(&self, current_token: &mut Option<Token>) -> Result<String> {
        Ok(match current_token {
            None => {
                let response = self.log_in()?;
                let access_token = response.access_token.clone();
                *current_token = Some(response);
                access_token
            }
            Some(token) => {
                if !token.is_unexpired() {
                    let new_token = self.refresh_token(&token.refresh_token)?;
                    let access_token = new_token.access_token.clone();
                    *current_token = Some(new_token);
                    access_token
                } else {
                    token.access_token.clone()
                }
            }
        })
    }

    fn log_in(&self) -> Result<Token> {
        debug!("Logging in…");
        let response = self
            .client
            .post(Url::parse_with_params(
                "https://auth.tado.com/oauth/token",
                &[
                    ("client_id", CLIENT_ID),
                    ("client_secret", CLIENT_SECRET),
                    ("grant_type", "password"),
                    ("scope", SCOPE),
                    ("username", &self.secrets.email),
                    ("password", &self.secrets.password),
                ],
            )?)
            .send()?
            .json::<Token>()?;
        debug!("Logged in, token expires at: {:?}", response.expires_at);
        Ok(response)
    }

    fn refresh_token(&self, refresh_token: &str) -> Result<Token> {
        debug!("Refreshing token…");
        let response = self
            .client
            .post(Url::parse_with_params(
                "https://auth.tado.com/oauth/token",
                &[
                    ("client_id", CLIENT_ID),
                    ("client_secret", CLIENT_SECRET),
                    ("grant_type", "refresh_token"),
                    ("scope", SCOPE),
                    ("refresh_token", refresh_token),
                ],
            )?)
            .send()?
            .json::<Token>()?;
        debug!("Logged in, token expires at: {:?}", response.expires_at);
        Ok(response)
    }

    fn call<U, R>(&self, url: U, access_token: &str) -> Result<R>
    where
        U: AsRef<str>,
        R: DeserializeOwned,
    {
        Ok(self
            .client
            .get(url.as_ref())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?
            .json()?)
    }

    fn get_me(&self, access_token: &str) -> Result<Me> {
        self.call("https://my.tado.com/api/v1/me", access_token)
    }

    fn get_home(&self, access_token: &str, home_id: u32) -> Result<Home> {
        self.call(format!("https://my.tado.com/api/v2/homes/{}", home_id), access_token)
    }

    fn get_zones(&self, access_token: &str, home_id: u32) -> Result<Zones> {
        self.call(
            format!("https://my.tado.com/api/v2/homes/{}/zones", home_id),
            access_token,
        )
    }

    fn get_weather(&self, access_token: &str, home_id: u32) -> Result<Weather> {
        self.call(
            format!("https://my.tado.com/api/v2/homes/{}/weather", home_id),
            access_token,
        )
    }

    fn get_home_state(&self, access_token: &str, home_id: u32) -> Result<HomeState> {
        self.call(
            format!("https://my.tado.com/api/v2/homes/{}/state", home_id),
            access_token,
        )
    }

    fn get_zone_state(&self, access_token: &str, home_id: u32, zone_id: u32) -> Result<ZoneState> {
        self.call(
            format!("https://my.tado.com/api/v2/homes/{}/zones/{}/state", home_id, zone_id),
            access_token,
        )
    }
}

#[derive(Deserialize)]
struct Token {
    access_token: String,

    #[serde(rename = "expires_in", deserialize_with = "deserialize_expires_at")]
    expires_at: SystemTime,

    refresh_token: String,
}

impl Token {
    pub fn is_unexpired(&self) -> bool {
        self.expires_at > SystemTime::now()
    }
}

#[derive(Deserialize)]
struct Me {
    #[serde(rename = "homeId")]
    home_id: u32,
}

#[derive(Deserialize)]
struct Home {
    name: String,
}

#[derive(Deserialize)]
struct Zone {
    id: u32,
    name: String,

    #[serde(rename = "type")]
    type_: ZoneType,
}

#[derive(Deserialize, PartialEq)]
enum ZoneType {
    #[serde(rename = "HEATING")]
    Heating,

    #[serde(rename = "HOT_WATER")]
    HotWater,
}

type Zones = Vec<Zone>;

#[derive(Deserialize)]
struct Weather {
    #[serde(rename = "solarIntensity")]
    solar_intensity: Percentage,
}

#[derive(Deserialize)]
struct Percentage {
    percentage: f64,
    timestamp: DateTime<Local>,
}

#[derive(Deserialize)]
struct HomeState {
    presence: Presence,
}

#[derive(Deserialize, PartialEq)]
enum Presence {
    #[serde(rename = "HOME")]
    Home,

    #[serde(rename = "AWAY")]
    Away,
}

#[derive(Deserialize)]
struct ZoneState {
    setting: ZoneSetting,
    link: Link,

    #[serde(rename = "sensorDataPoints")]
    sensor_data_points: SensorDataPoints,
}

#[derive(Deserialize)]
struct Link {
    state: LinkState,
}

#[derive(Deserialize, PartialEq)]
enum LinkState {
    #[serde(rename = "ONLINE")]
    Online,

    #[serde(rename = "OFFLINE")]
    Offline,
}

#[derive(Deserialize)]
struct ZoneSetting {
    power: PowerState,

    #[serde(flatten)]
    attributes: ZoneSettingAttributes,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ZoneSettingAttributes {
    #[serde(rename = "HOT_WATER")]
    HotWater,

    #[serde(rename = "HEATING")]
    Heating { temperature: ZoneTemperature },
}

#[derive(Deserialize, PartialEq)]
enum PowerState {
    #[serde(rename = "ON")]
    On,

    #[serde(rename = "OFF")]
    Off,
}

#[derive(Deserialize)]
struct ZoneTemperature {
    celsius: f64,

    #[allow(dead_code)]
    fahrenheit: f64,
}

#[derive(Deserialize)]
struct SensorDataPoints {
    #[serde(rename = "insideTemperature")]
    inside_temperature: Option<InsideTemperature>,

    humidity: Option<Percentage>,
}

#[derive(Deserialize)]
struct InsideTemperature {
    celsius: f64,
    timestamp: DateTime<Local>,
}

fn deserialize_expires_at<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<SystemTime, D::Error> {
    Ok(SystemTime::now() + Duration::from_secs(Deserialize::deserialize(deserializer)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token() -> Result<()> {
        // language=json
        serde_json::from_str::<Token>(
            r#"{"access_token": "abc", "token_type": "bearer", "refresh_token": "def", "expires_in": 599, "scope": "home.user", "jti": "xyz-123"}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_me() -> Result<()> {
        // language=json
        serde_json::from_str::<Me>(
            r#"{"name": "Terence Eden", "email": "you@example.com", "username": "your_user_name", "enabled": true, "id": "987654321", "homeId": 123456, "locale": "en_GB", "type": "WEB_USER"}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_home() -> Result<()> {
        // language=json
        serde_json::from_str::<Home>(
            r#"{"id": 123456, "name": " ", "dateTimeZone": "Europe/London", "dateCreated": "2015-12-18T19:21:59.315Z", "temperatureUnit": "CELSIUS", "installationCompleted": true, "partner": " ", "simpleSmartScheduleEnabled": true, "awayRadiusInMeters": 123.45, "usePreSkillsApps": true, "skills": [], "christmasModeEnabled": true, "contactDetails": {"name": "Terence Eden", "email": " ", "phone": " "}, "address": {"addressLine1": " ", "addressLine2": null, "zipCode": " ", "city": " ", "state": null, "country": "GBR"}, "geolocation": {"latitude": 12.3456789, "longitude": -1.23456}, "consentGrantSkippable": true}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_zones() -> Result<()> {
        // language=json
        serde_json::from_str::<Zones>(
            r#"[{"id": 1,"name": "Heating","type": "HEATING","dateCreated": "2015-12-21T15:46:45.000Z","deviceTypes": ["RU01"],"devices": [{"deviceType": "RU01","serialNo": " ","shortSerialNo": " ","currentFwVersion": "54.8","connectionState": {"value": true,"timestamp": "2019-02-13T19:30:52.733Z"},"characteristics": {"capabilities": ["INSIDE_TEMPERATURE_MEASUREMENT", "IDENTIFY", "OPEN_WINDOW_DETECTION"]},"batteryState": "NORMAL","duties": ["ZONE_UI", "ZONE_LEADER"]}],"reportAvailable": false,"supportsDazzle": true,"dazzleEnabled": true,"dazzleMode": {"supported": true,"enabled": true},"openWindowDetection": {"supported": true,"enabled": true,"timeoutInSeconds": 1800}}, {"id": 0,"name": "Hot Water","type": "HOT_WATER","dateCreated": "2016-10-03T11:31:42.272Z","deviceTypes": ["BU01", "RU01"],"devices": [{"deviceType": "BU01","serialNo": " ","shortSerialNo": " ","currentFwVersion": "49.4","connectionState": {"value": true,"timestamp": "2019-02-13T19:36:17.361Z"},"characteristics": {"capabilities": []},"isDriverConfigured": true,"duties": ["ZONE_DRIVER"]}, {"deviceType": "RU01","serialNo": " ","shortSerialNo": " ","currentFwVersion": "54.8","connectionState": {"value": true,"timestamp": "2019-02-13T19:30:52.733Z"},"characteristics": {"capabilities": ["INSIDE_TEMPERATURE_MEASUREMENT", "IDENTIFY", "OPEN_WINDOW_DETECTION"]},"batteryState": "NORMAL","duties": ["ZONE_UI", "ZONE_LEADER"]}],"reportAvailable": false,"supportsDazzle": false,"dazzleEnabled": false,"dazzleMode": {"supported": false},"openWindowDetection": {"supported": false}}]"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_home_state() -> Result<()> {
        // language=json
        serde_json::from_str::<HomeState>(r#"{"presence":"HOME"}"#)?;
        Ok(())
    }

    #[test]
    fn parse_zone_state_hot_water() -> Result<()> {
        // language=json
        serde_json::from_str::<ZoneState>(
            r#"{"tadoMode": "HOME","geolocationOverride": false,"geolocationOverrideDisableTime": null,"preparation": null,"setting": {"type": "HOT_WATER","power": "OFF","temperature": null},"overlayType": null,"overlay": null,"openWindow": null,"nextScheduleChange": {"start": "2019-02-13T19:00:00Z","setting": {"type": "HOT_WATER","power": "ON","temperature": null}},"link": {"state": "ONLINE"},"activityDataPoints": {},"sensorDataPoints": {}}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_zone_state_heating() -> Result<()> {
        // language=json
        serde_json::from_str::<ZoneState>(
            r#"{"tadoMode": "HOME","geolocationOverride": false,"geolocationOverrideDisableTime": null,"preparation": null,"setting": {"type": "HEATING","power": "ON","temperature": {"celsius": 15.00,"fahrenheit": 59.00}},"overlayType": null,"overlay": null,"openWindow": null,"nextScheduleChange": {"start": "2019-02-13T17:30:00Z","setting": {"type": "HEATING","power": "ON","temperature": {"celsius": 18.00,"fahrenheit": 64.40}}},"link": {"state": "ONLINE"},"activityDataPoints": {"heatingPower": {"type": "PERCENTAGE","percentage": 0.00,"timestamp": "2019-02-13T10:19:37.135Z"}},"sensorDataPoints": {"insideTemperature": {"celsius": 16.59,"fahrenheit": 61.86,"timestamp": "2019-02-13T10:30:52.733Z","type": "TEMPERATURE","precision": {"celsius": 0.1,"fahrenheit": 0.1}},"humidity": {"type": "PERCENTAGE","percentage": 57.20,"timestamp": "2019-02-13T10:30:52.733Z"}}}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_weather_ok() -> Result<()> {
        // language=json
        serde_json::from_str::<Weather>(
            r#"{"solarIntensity": {"type": "PERCENTAGE", "percentage": 68.10, "timestamp": "2019-02-10T10:35:00.989Z"}, "outsideTemperature": {"celsius": 8.00, "fahrenheit": 46.40, "timestamp": "2019-02-10T10:35:00.989Z", "type": "TEMPERATURE", "precision": {"celsius": 0.01, "fahrenheit": 0.01}}, "weatherState": {"type": "WEATHER_STATE", "value": "CLOUDY_PARTLY", "timestamp": "2019-02-10T10:35:00.989Z"}}"#,
        )?;
        Ok(())
    }
}
