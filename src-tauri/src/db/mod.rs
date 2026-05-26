pub mod migrations;
pub mod schema;

use std::path::Path;

use rusqlite::Connection;

pub fn init_db(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrations::run_migrations(&conn)?;
    Ok(conn)
}
