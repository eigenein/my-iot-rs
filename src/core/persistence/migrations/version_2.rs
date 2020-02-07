use crate::prelude::*;
use rusqlite::Connection;

pub fn migrate(db: &Connection) -> Result<()> {
    info!("Deleting all readings because of the changed serialization…");

    // language=sql
    db.execute_batch(
        r#"
            -- noinspection SqlWithoutWhere
            DELETE FROM readings;
            PRAGMA user_version = 2;
        "#,
    )?;
    Ok(())
}
