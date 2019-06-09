//! Database interface.
use crate::measurement::Measurement;

/// A database connection.
pub struct Db {
    connection: rusqlite::Connection,
}

/// Create a new database connection.
pub fn new() -> Db {
    let connection = rusqlite::Connection::open("my-iot.sqlite3").unwrap();

    #[rustfmt::skip]
    connection.execute("
        CREATE TABLE IF NOT EXISTS measurements (
            sensor TEXT NOT NULL,
            ts INTEGER NOT NULL,
            value BLOB NOT NULL
        );
    ", rusqlite::NO_PARAMS).unwrap();

    Db { connection }
}

impl Db {
    pub fn save_measurement(&self, measurement: &Measurement) {
        #[rustfmt::skip]
        self.connection.execute("
            INSERT INTO measurements (sensor, ts, value)
            VALUES (?1, ?2, ?3)
        ", &[
            &measurement.sensor as &rusqlite::ToSql,
            &measurement.timestamp.timestamp_millis(),
            &measurement.value,
        ]).unwrap();
    }

    // TODO: explain query plan select tag, max(ts) as ts, value from test group by tag;
}
