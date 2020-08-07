pub const MIGRATIONS: &[&str] = &[V1, V2, V3];

// language=sql
const V1: &str = r#"
    CREATE TABLE IF NOT EXISTS sensors (
        pk INTEGER NOT NULL PRIMARY KEY, -- `sensor_id` SeaHash
        sensor_id TEXT NOT NULL UNIQUE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        title TEXT DEFAULT NULL,
        room_title TEXT DEFAULT NULL, -- renamed to `location`
        value JSON NOT NULL,
        expires_at INTEGER NOT NULL -- unused
    );

    CREATE TABLE IF NOT EXISTS readings (
        sensor_fk INTEGER NOT NULL REFERENCES sensors ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        value JSON NOT NULL -- serialized `Value`
    );

    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp
        ON readings (sensor_fk ASC, timestamp DESC);

    CREATE TABLE IF NOT EXISTS user_data (
        pk TEXT NOT NULL PRIMARY KEY,
        value JSON NOT NULL,
        expires_at INTEGER NULL -- unix time, milliseconds
    );

    PRAGMA user_version = 1;
"#;

// language=sql
const V2: &str = r#"
    ALTER TABLE sensors ADD COLUMN is_writable INTEGER NOT NULL DEFAULT 0;
    PRAGMA user_version = 2;
"#;

// language=sql
const V3: &str = r#"
    DROP TABLE readings;
    DROP TABLE sensors;
    DROP TABLE user_data;

    CREATE TABLE IF NOT EXISTS sensors (
        pk INTEGER NOT NULL PRIMARY KEY, -- `sensor_id` SeaHash
        sensor_id TEXT NOT NULL UNIQUE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        title TEXT DEFAULT NULL,
        location TEXT DEFAULT NULL,
        value BLOB NOT NULL,
        is_writable INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS readings (
        sensor_fk INTEGER NOT NULL REFERENCES sensors ON UPDATE CASCADE ON DELETE CASCADE,
        timestamp INTEGER NOT NULL, -- unix time, milliseconds
        value JSON NOT NULL -- serialized `Value`
    );

    CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_fk_timestamp
        ON readings (sensor_fk ASC, timestamp DESC);

    CREATE TABLE IF NOT EXISTS user_data (
        pk TEXT NOT NULL PRIMARY KEY,
        value BLOB NOT NULL,
        expires_at INTEGER NULL -- unix time, milliseconds
    );

    PRAGMA user_version = 3;
"#;
