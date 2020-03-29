use crate::prelude::*;

pub const MIGRATIONS: &[fn(&Connection) -> Result<()>] = &[|_| Ok(()), migrate_1, migrate_2, migrate_3, migrate_4];

fn migrate_1(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            CREATE TABLE IF NOT EXISTS sensors (
                pk INTEGER PRIMARY KEY NOT NULL,
                sensor_id TEXT UNIQUE NOT NULL,
                timestamp DATETIME NOT NULL
            );

            -- Stores all sensor readings.
            CREATE TABLE IF NOT EXISTS readings (
                sensor_fk INTEGER NOT NULL REFERENCES sensors(pk) ON UPDATE CASCADE ON DELETE CASCADE,
                timestamp DATETIME NOT NULL,
                value BLOB NOT NULL
            );
            -- Descending index on `timestamp` is needed to speed up the select last queries.
            CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp ON readings (sensor_fk, timestamp DESC);
        "#,
    )?;
    Ok(())
}

fn migrate_2(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            -- noinspection SqlWithoutWhere
            DELETE FROM readings;
        "#,
    )?;
    Ok(())
}

fn migrate_3(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN title TEXT NULL DEFAULT NULL;
        "#,
    )?;
    Ok(())
}

fn migrate_4(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN room_title TEXT NULL DEFAULT NULL;
        "#,
    )?;
    Ok(())
}
