use crate::prelude::*;

pub const MIGRATIONS: &[fn(&Connection) -> Result<()>] = &[
    |_| Ok(()),
    create_initial_schema,
    drop_readings_because_of_changed_serialization_format,
    add_sensor_titles,
    add_room_titles,
    denormalize_actual_sensor_values,
];

fn create_initial_schema(db: &Connection) -> Result<()> {
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

fn drop_readings_because_of_changed_serialization_format(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            -- noinspection SqlWithoutWhere
            DELETE FROM readings;
        "#,
    )?;
    Ok(())
}

fn add_sensor_titles(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN title TEXT NULL DEFAULT NULL;
        "#,
    )?;
    Ok(())
}

fn add_room_titles(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN room_title TEXT NULL DEFAULT NULL;
        "#,
    )?;
    Ok(())
}

/// Denormalize `sensors` to avoid joining the `readings` table while
/// fetching actual sensor values. `Value::None` is set by default.
fn denormalize_actual_sensor_values(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN value BLOB NOT NULL DEFAULT x'81a14ec0'
        "#,
    )?;
    Ok(())
}
