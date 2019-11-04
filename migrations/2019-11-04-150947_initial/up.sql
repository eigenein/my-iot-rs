CREATE TABLE IF NOT EXISTS sensors (
    id INTEGER PRIMARY KEY NOT NULL,
    sensor TEXT UNIQUE NOT NULL,
    last_reading_id INTEGER NULL REFERENCES readings(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Stores all sensor readings.
CREATE TABLE IF NOT EXISTS readings (
    id INTEGER PRIMARY KEY NOT NULL,
    sensor_id INTEGER NOT NULL REFERENCES sensors(id) ON UPDATE CASCADE ON DELETE CASCADE,
    timestamp INTEGER NOT NULL,
    value BLOB NOT NULL
);
-- Descending index on `timestamp` is needed to speed up the select last queries.
CREATE UNIQUE INDEX IF NOT EXISTS readings_sensor_id_timestamp ON readings (sensor_id, timestamp DESC);
