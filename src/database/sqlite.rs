use rusqlite::{Connection, params};
use rusqlite::config::DbConfig;

pub fn open_in_memory() -> Connection {
    let conn = Connection::open_in_memory()?;
    let _ = &conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true);
    conn.execute("PRAGMA foreign_keys = true;", params![])?;

    conn
}