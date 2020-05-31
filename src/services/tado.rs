//! [tado°](https://www.tado.com/) API.

use crate::prelude::*;
use reqwest::blocking::Client;
use reqwest::Url;
use std::thread;
use std::time::Duration;

const CLIENT_ID: &str = "public-api-preview";
const CLIENT_SECRET: &str = "4HJGRffVR8xb3XdEUQpjgZ1VplJi6Xgw";
const SCOPE: &str = "home.user";
const REFRESH_PERIOD: Duration = Duration::from_millis(60000);

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
    pub fn spawn<'env>(&'env self, scope: &Scope<'env>, service_id: &'env str, bus: &mut Bus) -> Result<()> {
        let tx = bus.add_tx();
        let client = client_builder().build()?;

        debug!("Logging in…");
        self.login(&client)?;
        info!("Logged in");

        supervisor::spawn(scope, service_id, tx, move || -> Result<()> {
            loop {
                thread::sleep(REFRESH_PERIOD);
            }
        })
    }

    fn login(&self, client: &Client) -> Result<LoginResponse> {
        Ok(client
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
            .json::<LoginResponse>()?)
    }
}

#[derive(Deserialize)]
struct LoginResponse {
    pub access_token: String,

    #[serde(deserialize_with = "deserialize_duration")]
    pub expires_in: Duration,

    pub refresh_token: String,
}

fn deserialize_duration<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Duration, D::Error> {
    Ok(Duration::from_secs(Deserialize::deserialize(deserializer)?))
}
