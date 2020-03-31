use crate::prelude::*;
use crate::supervisor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn(db: &Arc<Mutex<Connection>>, bus: &mut Bus) -> Result<()> {
    let tx = bus.add_tx();
    let db = db.clone();

    supervisor::spawn("my-iot::db", tx.clone(), move || -> Result<()> {
        loop {
            let size = db.lock().unwrap().select_size()?;
            tx.send(
                Composer::new("db::size")
                    .value(Value::DataSize(size))
                    .title("Database Size".to_string())
                    .room_title("System".to_string())
                    .into(),
            )?;
            thread::sleep(INTERVAL);
        }
    })?;

    Ok(())
}
