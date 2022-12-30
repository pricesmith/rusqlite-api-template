use std::error::Error;
use std::path::Path;
use rusqlite::{params, Connection};
use barrel::backend::Sqlite;
use barrel::{types, Migration};
use crate::errors::ServerError;
use crate::util::now;
use super::context::{
    TLM_DB,
    TLM_LEVEL_0_TABLE,
    TLM_SINGLE_VALUE_TABLE,
}}

/// Up-to-date db
pub fn apply_all(conn: &Connection) -> Result<(), ServerError> {
    // apply migrations to tlm.db
    initial_tlm_db(conn)

    // apply migrations to other.db
    // etc
    // etc
}

/// Initial tlm.db migration
fn initial_tlm_db(conn: &Connection) -> Result<(), ServerError> {
    let mut m = Migration::new();

    create_initial_tlm_level_0_table(&mut m);
    create_initial_tlm_single_value_table(&mut m);

    conn.execute_batch(m.make::<Sqlite>().as_str()); // ?

    Ok(())
}

/// Creates the `level_0` table in the database.
fn create_initial_tlm_level_0_table(m: &mut Migration) {
    m.create_table(TLM_LEVEL_0_TABLE, |t| {
        t.add_column(
            "id",
            types::integer()
                .primary(true)
                .increments(true)
                .nullable(false),
        );
        t.add_column("uuid", types::text().nullable(false).unique(true));
        t.add_column("createdate", types::integer().nullable(false));
        t.add_column("metadata", types::text().nullable(false));
        t.add_column("packet", types::binary().nullable(false));
    });
}

/// Creates the `single_value` table in the database.
fn create_initial_tlm_single_value_table(m: &mut Migration) {
    m.create_table(TLM_SINGLE_VALUE_TABLE, |t| {
        t.add_column("name", types::text().nullable(false).unique(true));
        t.add_column("value", types::text().nullable(false));
    });
}