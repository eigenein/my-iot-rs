//! [YouLess](https://www.youless.nl/home.html) kWh meter to ethernet bridge.

use crate::prelude::*;
use crate::services::prelude::*;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct YouLess {
    #[serde(default = "default_interval_millis")]
    interval_millis: u64,

    #[serde(default = "default_host")]
    host: String,

    /// Which location the sensors should be put into.
    #[serde(default)]
    location: Option<String>,
}

/// Defaults to one minute.
const fn default_interval_millis() -> u64 {
    1000
}

fn default_host() -> String {
    "youless".into()
}

impl YouLess {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let url = format!("http://{}/e?f=j", self.host);
        let mut tx = bus.add_tx();

        task::spawn(async move {
            loop {
                handle_service_result(
                    &service_id,
                    Duration::from_millis(self.interval_millis),
                    self.loop_(&service_id, &url, &mut tx).await,
                )
                .await;
            }
        });

        Ok(())
    }

    async fn loop_(&self, service_id: &str, url: &str, tx: &mut Sender) -> Result {
        let response = CLIENT
            .get(url)
            .recv_json::<Vec<Response>>()
            .await
            .map_err(|err| anyhow!(err))?
            .pop()
            .ok_or_else(|| anyhow!("YouLess response is empty"))?;
        Message::new(format!("{}::nett", service_id))
            .value(Value::from_kwh(response.nett))
            .optional_location(self.location.clone())
            .sensor_title("Nett Counter")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::power", service_id))
            .value(Value::Power(response.power))
            .optional_location(self.location.clone())
            .sensor_title("Actual Consumption")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::consumption::low", service_id))
            .value(Value::from_kwh(response.consumption_low))
            .optional_location(self.location.clone())
            .sensor_title("Total Consumption Low")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::consumption::high", service_id))
            .value(Value::from_kwh(response.consumption_high))
            .optional_location(self.location.clone())
            .sensor_title("Total Consumption High")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::production::low", service_id))
            .value(Value::from_kwh(response.production_low))
            .optional_location(self.location.clone())
            .sensor_title("Total Production Low")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::production::high", service_id))
            .value(Value::from_kwh(response.production_high))
            .optional_location(self.location.clone())
            .sensor_title("Total Production High")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
        Message::new(format!("{}::gas", service_id))
            .value(Value::Volume(response.gas))
            .optional_location(self.location.clone())
            .sensor_title("Total Gas Consumption")
            .timestamp(response.timestamp)
            .send_to(tx)
            .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() -> Result {
        let response = serde_json::from_str::<Response>(
            r#"{"tm":1592815263,"net": 3602.148,"pwr":-368,"ts0":1584111000,"cs0": 0.000,"ps0": 0,"p1": 3851.282,"p2": 2949.180,"n1": 1000.784,"n2": 2197.530,"gas": 3564.538,"gts":2006221040}"#,
        )?;
        assert_eq!(response.timestamp, Utc.ymd(2020, 6, 22).and_hms(8, 41, 3));
        assert_eq!(response.gas, 3564.538);
        Ok(())
    }
}
