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
struct DevicesResponse {
    chimes: Vec<DeviceResponse>,
    doorbots: Vec<DeviceResponse>,
}

#[derive(Deserialize)]
struct DeviceResponse {
    #[allow(unused)]
    device_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() -> Result {
        serde_json::from_str::<DevicesResponse>(
            r#"{"authorized_doorbots": [], "base_stations": [], "beams": [], "beams_bridges": [], "chimes": [{"address": "Deleted", "alerts": {"connection": "online"}, "description": "Upstairs", "device_id": "247d4da73dd7", "do_not_disturb": {"seconds_left": 0}, "features": {"ringtones_enabled": true}, "firmware_version": "Up to Date", "id": 32335948, "kind": "chime", "latitude": 0.0, "location_id": "814aab1d-5a71-4e88-8ff9-eb11d5bab2a7", "longitude": 0.0, "owned": true, "owner": {"email": "john.doe@example.com", "first_name": "John", "id": 22363156, "last_name": "Doe"}, "ring_id": null, "settings": {"ding_audio_id": "00053e86-c08f-4aad-a500-84a51355433d", "ding_audio_user_id": "ring", "motion_audio_id": "00053e86-f43b-738f-7287-5693688ec52b", "motion_audio_user_id": "ring", "volume": 1}, "stolen": false, "time_zone": "Europe/Amsterdam"}], "doorbots": [{"address": "Deleted", "alerts": {"connection": "online"}, "battery_life": "77", "description": "Front Door", "device_id": "a8e2c131ebc8", "ext_power_state": 3, "external_connection": true, "features": {"motions_enabled": true, "rich_notifications_eligible": false, "show_24x7_lite": true, "show_offline_motion_events": false, "show_recordings": true, "show_vod_settings": true}, "firmware_version": "Up to Date", "id": 32333947, "kind": "doorbell_v3", "latitude": 0.0, "location_id": "814aab1d-5a71-4e88-8ff9-eb11d5bab2a7", "longitude": 0.0, "motion_snooze": null, "owned": true, "owner": {"email": "john.doe@example.com","first_name": "John","id": 22363156,"last_name": "Doe"},"ring_id": null,"settings": {"advanced_motion_detection_enabled": false,"advanced_motion_detection_human_only_mode": false,"advanced_motion_detection_types": [],"advanced_motion_zones": {"zone1": {"name": "Default Zone","state": 2,"vertex1": {"x": 0.0,"y": 0.4},"vertex2": {"x": 0.333333,"y": 0.4},"vertex3": {"x": 0.666666,"y": 0.4},"vertex4": {"x": 1.0,"y": 0.4},"vertex5": {"x": 1.0,"y": 1.0},"vertex6": {"x": 0.666666,"y": 1.0},"vertex7": {"x": 0.333333,"y": 1.0},"vertex8": {"x": 0.0,"y": 1.0}},"zone2": {"name": "Zone 2","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0},"vertex3": {"x": 0.0,"y": 0.0},"vertex4": {"x": 0.0,"y": 0.0},"vertex5": {"x": 0.0,"y": 0.0},"vertex6": {"x": 0.0,"y": 0.0},"vertex7": {"x": 0.0,"y": 0.0},"vertex8": {"x": 0.0,"y": 0.0}},"zone3": {"name": "Zone 3","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0},"vertex3": {"x": 0.0,"y": 0.0},"vertex4": {"x": 0.0,"y": 0.0},"vertex5": {"x": 0.0,"y": 0.0},"vertex6": {"x": 0.0,"y": 0.0},"vertex7": {"x": 0.0,"y": 0.0},"vertex8": {"x": 0.0,"y": 0.0}}},"advanced_motion_zones_enabled": false,"advanced_motion_zones_type": "6pir","advanced_pir_motion_zones": {"zone1_sensitivity": 5,"zone2_sensitivity": 5,"zone3_sensitivity": 5,"zone4_sensitivity": 5,"zone5_sensitivity": 5,"zone6_sensitivity": 5},"chime_settings": {"duration": 10,"enable": true,"type": 2},"doorbell_volume": 0,"enable_audio_recording": true,"enable_rich_notifications": false,"enable_vod": 1,"exposure_control": 2,"ignore_zones": {"zone1": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone2": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone3": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone4": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}}},"lite_24x7": {"enabled": true,"frequency_secs": 300,"resolution_p": 360,"subscribed": true},"live_view_disabled": false,"live_view_preset_profile": "middle","live_view_presets": ["low","middle","high","highest"],"loitering_threshold": 10,"motion_detection_enabled": true,"motion_snooze_preset_profile": "none","motion_snooze_presets": ["none","low","medium","high"],"motion_zones": [1,1,1,1,1],"offline_motion_event_settings": {"enabled": false,"frequency_after_secs": 2,"max_upload_kb": 5000,"period_after_secs": 30,"resolution_p": 360,"subscribed": true},"people_detection_eligible": true,"pir_sensitivity_1": 10,"recording_storage_type": "default_s3","rich_notifications_billing_eligible": true,"rich_notifications_face_crop_enabled": false,"rich_notifications_scene_source": "cloud","vod_status": "enabled","vod_suspended": 0},"stolen": false,"subscribed": false,"subscribed_motions": false,"time_zone": "Europe/Amsterdam"}],"other": [],"stickup_cams": []}"#,
        )?;
        Ok(())
    }
}
