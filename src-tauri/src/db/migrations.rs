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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations_fresh_db() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        run_migrations(&conn).expect("migrations should succeed on fresh db");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .expect("prepare statement");
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .expect("query tables")
            .filter_map(|r| r.ok())
            .collect();

        let expected = [
            "teams",
            "players",
            "plays",
            "pitches",
            "offense_transitions",
            "pitch_transitions",
            "pitcher_profiles",
            "games",
            "schema_version",
        ];
        for name in &expected {
            assert!(
                table_names.iter().any(|t| t == name),
                "expected table '{}' to exist, found: {:?}",
                name,
                table_names
            );
        }
    }

    #[test]
    fn test_run_migrations_idempotent() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        run_migrations(&conn).expect("first migration should succeed");
        run_migrations(&conn).expect("second migration should succeed without error");

        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
            .expect("query schema_version count");
        assert_eq!(row_count, 1, "schema_version should have exactly 1 row");

        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .expect("query schema_version value");
        assert_eq!(version, 1, "schema_version should be 1");
    }
}
