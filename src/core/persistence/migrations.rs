use crate::prelude::*;

pub const MIGRATIONS: &[fn(&rusqlite::Connection) -> Result<()>] = &[|_| Ok(()), create_initial_schema];

fn create_initial_schema(db: &rusqlite::Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            CREATE TABLE sensors (
                pk INTEGER NOT NULL PRIMARY KEY,
                sensor_id TEXT NOT NULL UNIQUE,
                timestamp DATETIME NOT NULL,
                title TEXT DEFAULT NULL,
                room_title TEXT DEFAULT NULL,
                value JSON NOT NULL,
                expires_at INTEGER NOT NULL
            );

            CREATE TABLE readings (
                sensor_fk INTEGER NOT NULL REFERENCES sensors ON UPDATE CASCADE ON DELETE CASCADE,
                timestamp DATETIME NOT NULL,
                value JSON NOT NULL
            );

            CREATE UNIQUE INDEX readings_sensor_fk_timestamp ON readings (sensor_fk ASC, timestamp DESC);
        "#,
    )?;
    Ok(())
}
