use crate::prelude::*;
use rusqlite::Connection;

pub fn migrate(db: &Connection) -> Result<()> {
    // language=sql
    db.execute_batch(
        r#"
            ALTER TABLE sensors ADD COLUMN title TEXT NULL DEFAULT NULL;
        "#,
    )?;
    Ok(())
}
