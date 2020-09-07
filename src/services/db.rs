//! System database service.

use crate::prelude::*;
use crate::services::prelude::*;

pub struct Db;

impl Db {
    pub fn spawn(self, service_id: String, bus: &mut Bus, db: Connection) {
        let mut tx = bus.add_tx();
        task::spawn(async move {
            loop {
                handle_service_result(&service_id, MINUTE, self.loop_(&db, &mut tx).await).await;
            }
        });
    }

    async fn loop_(&self, db: &Connection, tx: &mut Sender) -> Result {
        Message::new("db::size")
            .value(Value::DataSize(db.select_size().await?))
            .sensor_title("Database Size")
            .set_common_db_attributes()
            .send_to(tx)
            .await;
        Message::new("db::sensor_count")
            .value(Value::Counter(db.select_sensor_count().await?))
            .sensor_title("Sensor Count")
            .set_common_db_attributes()
            .send_to(tx)
            .await;
        Message::new("db::reading_count")
            .value(Value::Counter(db.select_total_reading_count().await?))
            .sensor_title("Reading Count")
            .set_common_db_attributes()
            .send_to(tx)
            .await;
        Ok(())
    }
}

impl Message {
    fn set_common_db_attributes(self) -> Self {
        self.location("System")
    }
}
