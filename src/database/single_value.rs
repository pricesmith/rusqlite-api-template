use rusqlite::{params, Result, Connection};

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    match conn.query_row(
        "select value from singlevalue where key = ?1",
        params![key],
        |row| Ok(row.get(0)?),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(x) => Err(x),
    }
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "insert into single_value (key, value) values (?1, ?2)
        on conflict (key) do update set value = ?2 where key = ?1",
        params![key, value],
    )?;
    Ok(())
}
