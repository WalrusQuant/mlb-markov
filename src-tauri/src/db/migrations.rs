use rusqlite::Connection;

use super::schema;

const CURRENT_VERSION: i32 = 1;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);",
    )?;

    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        schema::create_tables(conn)?;
        schema::create_indexes(conn)?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [CURRENT_VERSION],
        )?;
    }

    Ok(())
}
