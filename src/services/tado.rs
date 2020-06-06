//! [tado°](https://www.tado.com/) API.

use crate::prelude::*;
use reqwest::blocking::Client;
use reqwest::Url;
use std::thread;
use std::time::{Duration, SystemTime};

const CLIENT_ID: &str = "public-api-preview";
const CLIENT_SECRET: &str = "4HJGRffVR8xb3XdEUQpjgZ1VplJi6Xgw";
const SCOPE: &str = "home.user";
const REFRESH_PERIOD: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Tado {
    pub secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Secrets {
    pub email: String,
    pub password: String,
}

impl Tado {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let client = client_builder().build()?;
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut token: Option<Token> = None;
            loop {
                if let Err(error) = self.loop_(&service_id, &client, &mut token, &tx) {
                    error!("Failed to refresh the sensors: {}", error.to_string());
                }
                thread::sleep(REFRESH_PERIOD);
            }
        })?;

        Ok(())
    }

    fn loop_(&self, service_id: &str, client: &Client, token: &mut Option<Token>, tx: &Sender) -> Result<()> {
        let access_token = self.check_login(&client, token)?;
        let ttl = chrono::Duration::seconds(120);

        let me = self.get_me(&client, &access_token)?;
        let home = self.get_home(&client, &access_token, me.home_id)?;
        let weather = self.get_weather(&client, &access_token, me.home_id)?;

        if let SolarIntensity::Percentage(percentage) = weather.solar_intensity {
            Message::new(format!("{}::{}::solar_intensity", service_id, me.home_id))
                .timestamp(percentage.timestamp)
                .expires_in(ttl)
                .value(Value::RelativeIntensity(percentage.percentage))
                .room_title(&home.name)
                .sensor_title("Solar Intensity")
                .send_and_forget(&tx);
        }

        Ok(())
    }

    /// Checks if the service is logged in. Logs in or refreshes access token when needed.
    /// Returns an active access token.
    fn check_login(&self, client: &Client, current_token: &mut Option<Token>) -> Result<String> {
        Ok(match current_token {
            None => {
                let response = self.log_in(&client)?;
                let access_token = response.access_token.clone();
                *current_token = Some(response);
                access_token
            }
            Some(token) => {
                if !token.is_unexpired() {
                    let new_token = self.refresh_token(&client, &token.refresh_token)?;
                    let access_token = new_token.access_token.clone();
                    *current_token = Some(new_token);
                    access_token
                } else {
                    token.access_token.clone()
                }
            }
        })
    }

    fn log_in(&self, client: &Client) -> Result<Token> {
        debug!("Logging in…");
        let response = client
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

    fn refresh_token(&self, client: &Client, refresh_token: &str) -> Result<Token> {
        debug!("Refreshing token…");
        let response = client
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

    fn get_me(&self, client: &Client, access_token: &str) -> Result<Me> {
        Ok(client
            .get("https://my.tado.com/api/v1/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?
            .json()?)
    }

    fn get_home(&self, client: &Client, access_token: &str, home_id: u32) -> Result<Home> {
        Ok(client
            .get(&format!("https://my.tado.com/api/v2/homes/{}", home_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?
            .json()?)
    }

    #[allow(dead_code)]
    fn get_zones(&self, client: &Client, access_token: &str, home_id: u32) -> Result<Vec<Zone>> {
        Ok(client
            .get(&format!("https://my.tado.com/api/v2/homes/{}/zones", home_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?
            .json()?)
    }

    fn get_weather(&self, client: &Client, access_token: &str, home_id: u32) -> Result<Weather> {
        Ok(client
            .get(&format!("https://my.tado.com/api/v2/homes/{}/weather", home_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()?
            .json()?)
    }
}

#[derive(Deserialize)]
struct Token {
    pub access_token: String,

    #[serde(rename = "expires_in", deserialize_with = "deserialize_expires_at")]
    pub expires_at: SystemTime,

    pub refresh_token: String,
}

impl Token {
    pub fn is_unexpired(&self) -> bool {
        self.expires_at > SystemTime::now()
    }
}

#[derive(Deserialize)]
struct Me {
    #[serde(rename = "homeId")]
    pub home_id: u32,

    pub name: String,
}

#[derive(Deserialize)]
struct Home {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Zone {
    #[serde(rename = "HEATING")]
    Heating(HeatingZone),

    #[serde(rename = "HOT_WATER")]
    HotWater(HotWaterZone),

    #[serde(other)]
    NotImplemented,
}

#[derive(Deserialize)]
struct HeatingZone {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
struct HotWaterZone {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
struct Weather {
    #[serde(rename = "solarIntensity")]
    pub solar_intensity: SolarIntensity,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum SolarIntensity {
    #[serde(rename = "PERCENTAGE")]
    Percentage(Percentage),

    #[serde(other)]
    NotImplemented,
}

#[derive(Deserialize)]
struct Percentage {
    pub percentage: f64,
    pub timestamp: DateTime<Local>,
}

fn deserialize_expires_at<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<SystemTime, D::Error> {
    Ok(SystemTime::now() + Duration::from_secs(Deserialize::deserialize(deserializer)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_heating_zone_ok() -> Result<()> {
        serde_json::from_str::<Zone>(
            r#"{"id": 1, "name": "Heating", "type": "HEATING", "dateCreated": "2015-12-21T15:46:45.000Z", "deviceTypes": ["RU01"], "devices": [{"deviceType": "RU01", "serialNo": " ", "shortSerialNo": " ", "currentFwVersion": "54.8", "connectionState": {"value": true, "timestamp": "2019-02-13T19:30:52.733Z"}, "characteristics": {"capabilities": ["INSIDE_TEMPERATURE_MEASUREMENT", "IDENTIFY", "OPEN_WINDOW_DETECTION"]}, "batteryState": "NORMAL", "duties": ["ZONE_UI", "ZONE_LEADER"]}], "reportAvailable": false, "supportsDazzle": true, "dazzleEnabled": true, "dazzleMode": {"supported": true, "enabled": true}, "openWindowDetection": {"supported": true, "enabled": true, "timeoutInSeconds": 1800}}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_hot_water_zone_ok() -> Result<()> {
        serde_json::from_str::<Zone>(
            r#"{"id": 0, "name": "Hot Water", "type": "HOT_WATER", "dateCreated": "2016-10-03T11:31:42.272Z", "deviceTypes": ["BU01", "RU01"], "devices": [{"deviceType": "BU01", "serialNo": " ", "shortSerialNo": " ", "currentFwVersion": "49.4", "connectionState": {"value": true, "timestamp": "2019-02-13T19:36:17.361Z"}, "characteristics": {"capabilities": []}, "isDriverConfigured": true, "duties": ["ZONE_DRIVER"]}, {"deviceType": "RU01", "serialNo": " ", "shortSerialNo": " ", "currentFwVersion": "54.8", "connectionState": {"value": true, "timestamp": "2019-02-13T19:30:52.733Z"}, "characteristics": {"capabilities": ["INSIDE_TEMPERATURE_MEASUREMENT", "IDENTIFY", "OPEN_WINDOW_DETECTION"]}, "batteryState": "NORMAL", "duties": ["ZONE_UI", "ZONE_LEADER"]}], "reportAvailable": false, "supportsDazzle": false, "dazzleEnabled": false, "dazzleMode": {"supported": false}, "openWindowDetection": {"supported": false}}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_weather_ok() -> Result<()> {
        serde_json::from_str::<Weather>(
            r#"{"solarIntensity": {"type": "PERCENTAGE", "percentage": 68.10, "timestamp": "2019-02-10T10:35:00.989Z"}, "outsideTemperature": {"celsius": 8.00, "fahrenheit": 46.40, "timestamp": "2019-02-10T10:35:00.989Z", "type": "TEMPERATURE", "precision": {"celsius": 0.01, "fahrenheit": 0.01}}, "weatherState": {"type": "WEATHER_STATE", "value": "CLOUDY_PARTLY", "timestamp": "2019-02-10T10:35:00.989Z"}}"#,
        )?;
        Ok(())
    }
}
