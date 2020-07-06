use crate::prelude::*;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Ring {
    initial_refresh_token: String,

    #[serde(default = "default_interval_millis")]
    interval_millis: u64,
}

const fn default_interval_millis() -> u64 {
    60000
}

impl Ring {
    pub fn spawn(self, service_id: String, bus: &mut Bus, _db: &Connection) -> Result {
        let _tx = bus.add_tx();
        spawn_service_loop(
            service_id,
            Duration::from_millis(self.interval_millis),
            move || unimplemented!(),
        )
    }
}
