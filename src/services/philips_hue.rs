use futures::pin_mut;

use crate::prelude::*;
use crate::services::prelude::*;
use async_std::net::IpAddr;
use mdns::{Record, RecordKind};

const SERVICE_NAME: &str = "_hue._tcp.local";

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct PhilipsHue {
    #[serde(default = "default_discovery_interval_secs")]
    discovery_interval_secs: u64,

    #[serde(skip, default = "Vec::new")]
    bridges: Vec<Bridge>,
}

const fn default_discovery_interval_secs() -> u64 {
    300
}

#[derive(Deserialize, Debug, Clone)]
struct Bridge {
    #[serde(rename = "bridgeid")]
    id: String,
}

impl PhilipsHue {
    pub fn spawn(self, service_id: String, bus: &mut Bus) -> Result {
        let mut tx = bus.add_tx();

        task::spawn(async move {
            loop {
                handle_service_result(&service_id, MINUTE, self.discover(&service_id, &mut tx).await).await;
            }
        });

        Ok(())
    }

    async fn discover(&self, _service_id: &str, _tx: &mut Sender) -> Result {
        let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(self.discovery_interval_secs))?.listen();
        pin_mut!(stream);

        while let Some(Ok(response)) = stream.next().await {
            if let Some(ip_addr) = response.records().filter_map(to_ip_addr).next() {
                self.on_bridge_discovered(ip_addr).await;
            }
        }

        Err(Error::new("the mDNS discovery stream has unexpectedly ended"))
    }

    async fn on_bridge_discovered(&self, ip_addr: IpAddr) {
        info!("Bridge discovered: {}.", ip_addr);
    }
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(ip_addr) => Some(ip_addr.into()),
        RecordKind::AAAA(ip_addr) => Some(ip_addr.into()),
        _ => None,
    }
}
