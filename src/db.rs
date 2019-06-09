//! Database interface.
use sqlite::{Connection, open};

pub fn new() -> Connection {
    let connection = open("my-iot.sqlite3").unwrap();
    connection.execute("
        CREATE TABLE IF NOT EXISTS measurements (
            sensor TEXT NOT NULL,
            ts FLOAT NOT NULL,
            value BLOB NOT NULL
        );
    ").unwrap();
    connection
}
