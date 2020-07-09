use std::str::FromStr;
use std::time::Duration;

use reqwest::Method;

use crate::prelude::*;
use crate::services::{call_json_api, CLIENT};
use bytes::Bytes;

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
        let tx = bus.add_tx();
        let db = db.clone();

        spawn_service_loop(
            service_id.clone(),
            Duration::from_millis(self.interval_millis),
            move || {
                let devices = self.get_devices(&service_id, &db)?;
                info!("{} doorbots, {} chimes.", devices.doorbots.len(), devices.chimes.len());
                for device in devices.doorbots {
                    if let Some(ref battery_life) = device.battery_life {
                        Message::new(format!("{}::doorbot::{}::battery_life", service_id, device.id))
                            .location(&device.description)
                            .sensor_title("Doorbot Battery State")
                            .value(Value::BatteryLife(f64::from_str(battery_life)?))
                            .send_and_forget(&tx);
                    }
                    self.process_doorbot_recordings(&service_id, &db, &device, &tx)?;
                }
                Ok(())
            },
        )
    }

    fn process_doorbot_recordings(
        &self,
        service_id: &str,
        db: &Connection,
        device: &DeviceResponse,
        tx: &Sender,
    ) -> Result {
        let response = self.get_doorbot_history(service_id, db, device.id)?;
        for history in response.iter() {
            let flag_key = format!("{}::doorbot::history::{}::is_processed", service_id, history.id);
            if db.get_user_data(&flag_key)? == Some(true) {
                debug!("[{}] Recording #{} has already been processed.", service_id, history.id);
                continue;
            }
            if history.recording.status != Some(RecordingStatus::Ready) {
                warn!("[{}] Recording #{} is not ready yet.", service_id, history.id);
                continue;
            }
            let content = self.get_recording(service_id, db, history)?;
            info!("[{}] {} bytes downloaded.", service_id, content.len());
            Message::new(format!(
                "{}::doorbot::{}::recording::{}",
                service_id, device.id, history.id
            ))
            .type_(MessageType::ReadNonLogged)
            .timestamp(history.created_at)
            .sensor_title("Recording")
            .location(&device.description)
            .value(Value::Blob(Arc::new(content)))
            .send_and_forget(tx);
            db.set_user_data(&flag_key, true, None)?;
        }
        Ok(())
    }
}

/// Ring.com APIs.
impl Ring {
    fn get_devices(&self, service_id: &str, db: &Connection) -> Result<DevicesResponse> {
        call_json_api(
            Method::GET,
            &self.get_access_token(service_id, db)?,
            "https://api.ring.com/clients_api/ring_devices",
        )
    }

    fn get_doorbot_history(&self, service_id: &str, db: &Connection, device_id: i32) -> Result<Vec<HistoryResponse>> {
        call_json_api(
            Method::GET,
            &self.get_access_token(service_id, db)?,
            &format!("https://api.ring.com/clients_api/doorbots/{}/history", device_id),
        )
    }

    fn get_recording(&self, service_id: &str, db: &Connection, history: &HistoryResponse) -> Result<Bytes> {
        info!("[{}] Downloading recording #{}…", service_id, history.id);
        Ok(CLIENT
            .get(&format!(
                "https://api.ring.com/clients_api/dings/{}/recording",
                history.id
            ))
            .header(
                "Authorization",
                format!("Bearer {}", self.get_access_token(service_id, db)?),
            )
            .send()?
            .error_for_status()?
            .bytes()?)
    }
}

/// Authentication.
impl Ring {
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
    /// Example: `32335948`.
    id: i32,

    /// Example: `"Upstairs"`.
    description: String,

    /// Example: `"77"`.
    battery_life: Option<String>,
}

#[derive(Deserialize)]
struct HistoryResponse {
    /// Example: `6846739887801852027`.
    id: i64,

    recording: RecordingResponse,

    created_at: DateTime<Local>,
}

#[derive(Deserialize)]
struct RecordingResponse {
    /// Seems to be `null` in an unready status.
    status: Option<RecordingStatus>,
}

#[derive(Deserialize, PartialEq)]
enum RecordingStatus {
    #[serde(rename = "ready")]
    Ready,

    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_devices_ok() -> Result {
        serde_json::from_str::<DevicesResponse>(
            r#"{"authorized_doorbots": [], "base_stations": [], "beams": [], "beams_bridges": [], "chimes": [{"address": "Deleted", "alerts": {"connection": "online"}, "description": "Upstairs", "device_id": "247d4da73dd7", "do_not_disturb": {"seconds_left": 0}, "features": {"ringtones_enabled": true}, "firmware_version": "Up to Date", "id": 32335948, "kind": "chime", "latitude": 0.0, "location_id": "814aab1d-5a71-4e88-8ff9-eb11d5bab2a7", "longitude": 0.0, "owned": true, "owner": {"email": "john.doe@example.com", "first_name": "John", "id": 22363156, "last_name": "Doe"}, "ring_id": null, "settings": {"ding_audio_id": "00053e86-c08f-4aad-a500-84a51355433d", "ding_audio_user_id": "ring", "motion_audio_id": "00053e86-f43b-738f-7287-5693688ec52b", "motion_audio_user_id": "ring", "volume": 1}, "stolen": false, "time_zone": "Europe/Amsterdam"}], "doorbots": [{"address": "Deleted", "alerts": {"connection": "online"}, "battery_life": "77", "description": "Front Door", "device_id": "a8e2c131ebc8", "ext_power_state": 3, "external_connection": true, "features": {"motions_enabled": true, "rich_notifications_eligible": false, "show_24x7_lite": true, "show_offline_motion_events": false, "show_recordings": true, "show_vod_settings": true}, "firmware_version": "Up to Date", "id": 32333947, "kind": "doorbell_v3", "latitude": 0.0, "location_id": "814aab1d-5a71-4e88-8ff9-eb11d5bab2a7", "longitude": 0.0, "motion_snooze": null, "owned": true, "owner": {"email": "john.doe@example.com","first_name": "John","id": 22363156,"last_name": "Doe"},"ring_id": null,"settings": {"advanced_motion_detection_enabled": false,"advanced_motion_detection_human_only_mode": false,"advanced_motion_detection_types": [],"advanced_motion_zones": {"zone1": {"name": "Default Zone","state": 2,"vertex1": {"x": 0.0,"y": 0.4},"vertex2": {"x": 0.333333,"y": 0.4},"vertex3": {"x": 0.666666,"y": 0.4},"vertex4": {"x": 1.0,"y": 0.4},"vertex5": {"x": 1.0,"y": 1.0},"vertex6": {"x": 0.666666,"y": 1.0},"vertex7": {"x": 0.333333,"y": 1.0},"vertex8": {"x": 0.0,"y": 1.0}},"zone2": {"name": "Zone 2","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0},"vertex3": {"x": 0.0,"y": 0.0},"vertex4": {"x": 0.0,"y": 0.0},"vertex5": {"x": 0.0,"y": 0.0},"vertex6": {"x": 0.0,"y": 0.0},"vertex7": {"x": 0.0,"y": 0.0},"vertex8": {"x": 0.0,"y": 0.0}},"zone3": {"name": "Zone 3","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0},"vertex3": {"x": 0.0,"y": 0.0},"vertex4": {"x": 0.0,"y": 0.0},"vertex5": {"x": 0.0,"y": 0.0},"vertex6": {"x": 0.0,"y": 0.0},"vertex7": {"x": 0.0,"y": 0.0},"vertex8": {"x": 0.0,"y": 0.0}}},"advanced_motion_zones_enabled": false,"advanced_motion_zones_type": "6pir","advanced_pir_motion_zones": {"zone1_sensitivity": 5,"zone2_sensitivity": 5,"zone3_sensitivity": 5,"zone4_sensitivity": 5,"zone5_sensitivity": 5,"zone6_sensitivity": 5},"chime_settings": {"duration": 10,"enable": true,"type": 2},"doorbell_volume": 0,"enable_audio_recording": true,"enable_rich_notifications": false,"enable_vod": 1,"exposure_control": 2,"ignore_zones": {"zone1": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone2": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone3": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}},"zone4": {"name": "undefined","state": 0,"vertex1": {"x": 0.0,"y": 0.0},"vertex2": {"x": 0.0,"y": 0.0}}},"lite_24x7": {"enabled": true,"frequency_secs": 300,"resolution_p": 360,"subscribed": true},"live_view_disabled": false,"live_view_preset_profile": "middle","live_view_presets": ["low","middle","high","highest"],"loitering_threshold": 10,"motion_detection_enabled": true,"motion_snooze_preset_profile": "none","motion_snooze_presets": ["none","low","medium","high"],"motion_zones": [1,1,1,1,1],"offline_motion_event_settings": {"enabled": false,"frequency_after_secs": 2,"max_upload_kb": 5000,"period_after_secs": 30,"resolution_p": 360,"subscribed": true},"people_detection_eligible": true,"pir_sensitivity_1": 10,"recording_storage_type": "default_s3","rich_notifications_billing_eligible": true,"rich_notifications_face_crop_enabled": false,"rich_notifications_scene_source": "cloud","vod_status": "enabled","vod_suspended": 0},"stolen": false,"subscribed": false,"subscribed_motions": false,"time_zone": "Europe/Amsterdam"}],"other": [],"stickup_cams": []}"#,
        )?;
        Ok(())
    }

    #[test]
    fn parse_history_ok() -> Result {
        serde_json::from_str::<Vec<HistoryResponse>>(
            r#"[{"id":6846804621548937339,"created_at":"2020-07-07T18:18:47.000Z","answered":false,"events":[],"kind":"ding","favorite":false,"snapshot_url":"","recording":{"status":null},"duration":32.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846802774713000059,"created_at":"2020-07-07T18:11:37.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":27.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846785319965909115,"created_at":"2020-07-07T17:03:53.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846784585526501499,"created_at":"2020-07-07T17:01:02.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":23.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846780943394234491,"created_at":"2020-07-07T16:46:54.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846778280514510971,"created_at":"2020-07-07T16:36:34.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":20.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846772740006699131,"created_at":"2020-07-07T16:15:04.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":23.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846770416429391995,"created_at":"2020-07-07T16:06:03.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846769647630246011,"created_at":"2020-07-07T16:03:04.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846769402817110139,"created_at":"2020-07-07T16:02:07.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846767010520326267,"created_at":"2020-07-07T15:52:50.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":25.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846764635403411579,"created_at":"2020-07-07T15:43:37.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846762303236169851,"created_at":"2020-07-07T15:34:34.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846761684760879227,"created_at":"2020-07-07T15:32:10.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":21.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846759747730628731,"created_at":"2020-07-07T15:24:39.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846758673988804731,"created_at":"2020-07-07T15:20:29.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":30.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846756814267965563,"created_at":"2020-07-07T15:13:16.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":23.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846756522210189435,"created_at":"2020-07-07T15:12:08.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":20.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846754314596999291,"created_at":"2020-07-07T15:03:34.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":20.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}},{"id":6846752824243347579,"created_at":"2020-07-07T14:57:47.000Z","answered":false,"events":[],"kind":"motion","favorite":false,"snapshot_url":"","recording":{"status":"ready"},"duration":22.0,"cv_properties":{"person_detected":null,"stream_broken":null,"detection_type":null}}]"#,
        )?;
        Ok(())
    }
}
