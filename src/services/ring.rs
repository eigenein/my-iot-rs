use std::time::Duration;

use crate::prelude::*;
use crate::services::CLIENT;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Ring {
    #[serde(default = "default_interval_millis")]
    interval_millis: u64,

    secrets: Secrets,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct Secrets {
    /// Initial `refresh_token` used to get an active access token for the first time.
    initial_refresh_token: String,
}

const fn default_interval_millis() -> u64 {
    60000
}

impl Ring {
    pub fn spawn(self, service_id: String, bus: &mut Bus, db: &Connection) -> Result {
        let _tx = bus.add_tx();
        let db = db.clone();

        spawn_service_loop(
            service_id.clone(),
            Duration::from_millis(self.interval_millis),
            move || {
                self.get_access_token(&service_id, &db)?;
                Ok(())
            },
        )
    }

    /// Gets an active access token. Refreshes an old token, if needed.
    fn get_access_token(&self, service_id: &str, db: &Connection) -> Result<String> {
        let access_token_key = format!("{}::access_token", service_id);
        let refresh_token_key = format!("{}::refresh_token", service_id);

        Ok(match db.get_user_data::<String>(&access_token_key)? {
            Some(access_token) => {
                debug!("[{}] Found an existing access token.", service_id);
                access_token
            }
            None => {
                info!("[{}] Refreshing access token…", service_id);
                let refresh_token = db
                    .get_user_data::<String>(&refresh_token_key)?
                    .unwrap_or_else(|| self.secrets.initial_refresh_token.clone());
                let response = CLIENT
                    .post("https://oauth.ring.com/oauth/token")
                    .form(&[
                        ("scope", "client"),
                        ("client_id", "ring_official_android"),
                        ("grant_type", "refresh_token"),
                        ("refresh_token", &refresh_token),
                    ])
                    .send()?
                    .error_for_status()?
                    .json::<TokenResponse>()?;
                db.set_user_data(
                    &access_token_key,
                    &response.access_token,
                    Some(Local::now() + chrono::Duration::seconds(response.expires_in)),
                )?;
                db.set_user_data(&refresh_token_key, &response.refresh_token, None)?;
                info!("[{}] Got a new access token.", service_id);
                response.access_token
            }
        })
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[allow(unused)]
#[derive(Deserialize)]
struct DevicesResponse {}
