//! [YouLess](https://www.youless.nl/home.html) kWh meter to ethernet bridge.

use crate::prelude::*;
use crate::services::prelude::*;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct YouLess {
    #[serde(default = "default_interval_millis")]
    interval_millis: u32,

    #[serde(default = "default_url")]
    url: String,

    #[serde(default = "default_room_title")]
    room_title: String,

    #[serde(skip, default = "default_client")]
    client: Client,
}

/// Defaults to one minute.
const fn default_interval_millis() -> u32 {
    1000
}

fn default_url() -> String {
    "http://youless/e?f=j".into()
}

fn default_room_title() -> String {
    "Home".into()
}

impl YouLess {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result<()> {
        let interval = Duration::from_millis(self.interval_millis as u64);
        let ttl = chrono::Duration::milliseconds((self.interval_millis as i64) * 2);
        let tx = bus.add_tx();

        thread::Builder::new().name(service_id.clone()).spawn(move || loop {
            if let Err(error) = self.loop_(&service_id, &tx, ttl) {
                error!("Failed to refresh the sensors: {}", error.to_string());
            }
            thread::sleep(interval);
        })?;

        Ok(())
    }

    fn loop_(&self, service_id: &str, tx: &Sender, ttl: chrono::Duration) -> Result<()> {
        let response = self
            .client
            .get(&self.url)
            .send()?
            .json::<Vec<Response>>()?
            .pop()
            .ok_or("YouLess response is empty")?;
        Message::new(format!("{}::nett", service_id))
            .value(Value::from_kwh(response.nett))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Nett Counter")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::power", service_id))
            .value(Value::Power(response.power))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Actual Consumption")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::consumption_low", service_id))
            .value(Value::from_kwh(response.consumption_low))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Total Consumption Low")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::consumption_high", service_id))
            .value(Value::from_kwh(response.consumption_high))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Total Consumption High")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::production_low", service_id))
            .value(Value::from_kwh(response.production_low))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Total Production Low")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::production_high", service_id))
            .value(Value::from_kwh(response.production_high))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Total Production High")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Message::new(format!("{}::consumption_gas", service_id))
            .value(Value::Volume(response.gas))
            .expires_in(ttl)
            .room_title(&self.room_title)
            .sensor_title("Total Gas Consumption")
            .timestamp(response.timestamp)
            .send_and_forget(tx);
        Ok(())
    }
}

/// http://wiki.td-er.nl/index.php?title=YouLess#Enelogic_.28default.29_firmware
#[derive(Deserialize)]
struct Response {
    #[serde(rename = "tm", deserialize_with = "deserialize_timestamp")]
    timestamp: DateTime<Local>,

    /// Netto counter, as displayed in the web-interface of the LS-120.
    /// It seems equal to: `p1 + p2 - n1 - n2` Perhaps also includes some user set offset.
    #[serde(rename = "net")]
    nett: f64,

    /// Actual power use in Watt (can be negative).
    #[serde(rename = "pwr")]
    power: f64,

    /// P1 consumption counter (low tariff).
    #[serde(rename = "p1")]
    consumption_low: f64,

    /// P2 consumption counter (high tariff).
    #[serde(rename = "p2")]
    consumption_high: f64,

    /// N1 production counter (low tariff).
    #[serde(rename = "n1")]
    production_low: f64,

    /// N2 production counter (high tariff).
    #[serde(rename = "n2")]
    production_high: f64,

    /// Counter gas-meter (in m^3).
    #[serde(rename = "gas")]
    gas: f64,
}

fn deserialize_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> StdResult<DateTime<Local>, D::Error> {
    Ok(Local.timestamp(i64::deserialize(deserializer)?, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn parse() -> Result<()> {
        let response = serde_json::from_str::<Response>(
            r#"{"tm":1592815263,"net": 3602.148,"pwr":-368,"ts0":1584111000,"cs0": 0.000,"ps0": 0,"p1": 3851.282,"p2": 2949.180,"n1": 1000.784,"n2": 2197.530,"gas": 3564.538,"gts":2006221040}"#,
        )?;
        assert_eq!(response.timestamp, Utc.ymd(2020, 6, 22).and_hms(8, 41, 3));
        assert_eq!(response.gas, 3564.538);
        Ok(())
    }
}
