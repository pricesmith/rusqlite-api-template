// use std::io::prelude::*;
// use std::collections::HashMap;
use std::sync::{Mutex, Arc, /* RwLock */};
use std::{fs, time::Duration};
use std::error::Error;
use rusqlite::{Statement, Result};
use rusqlite::Connection as RusqliteConnection;
use crate::errors::ServerError;
use super::{migrations, single_value, connection::ConnectionPool};

/// tlm.db
pub const TLM_DB: &str = "tlm.db";
/// tlm tables
pub const TLM_LEVEL_0_TABLE: &str               = "level_0";
pub const TLM_SINGLE_VALUE_TABLE: &str          = "single_value"; // ?: Is this just for level 0 tlm?

/// tlm_test.db
pub const TLM_TEST_DB: &str = "tlm_test.db";
/// tlm_test tables
pub const TLM_TEST_LEVEL_0_TABLE: &str          = "level_0";
pub const TLM_TEST_SINGLE_VALUE_TABLE: &str     = "single_value";

/// tlm_cache.db <- ex: when we've no API connection, we store locally to the tlm_cache
pub const TLM_CACHE: &str = "tlm_cache.db";
/// tlm_cache tables
pub const TLM_CACHE_LEVEL_0_TABLE: &str         = "level_0";
pub const TLM_CACHE_SINGLE_VALUE_TABLE: &str    = "single_value";

/// mem db
pub const MEM_DB: &str = ":memory:"; // Maybe in-mem cache?
/// mem db tables
// ...
// ...

///
// const NUM_DB: usize = 4;

fn read_sql_from_file(path: &str) -> String {
    fs::read_to_string(path).expect(r#"Failed to read SQL file."#)
}

pub struct DbContext<'a> {
    pub conn_pool: ConnectionPool,
    // pub db_map: HashMap<String, Arc<RwLock<RusqliteConnection>>>,

    /// level_0_packet
    get_all_level_0_packets_statement: Option<Statement<'a>>,
    get_last_level_0_packet_statement: Option<Statement<'a>>,

    /// single_value
    get_all_single_values_statement: Option<Statement<'a>>,
    get_last_single_value_statement: Option<Statement<'a>>,
}

impl<'a> DbContext<'a> {
    pub fn new() -> Self {        
        DbContext { 
            conn_pool: ConnectionPool::new(10, Duration::from_secs(30)),
            // db_map: HashMap::with_capacity(NUM_DB),

            // level_0_packet
            get_all_level_0_packets_statement: None,
            get_last_level_0_packet_statement: None,

            // single_value
            // init_single_value_statement: None,
            get_all_single_values_statement: None,
            get_last_single_value_statement: None,
        }
    }

    pub fn init(&mut self) {
        let &mut pool = &self.conn_pool;

        // iterate through all dbs and initialize
        self.init_db(pool, TLM_DB);
    }

    fn init_db(&mut self, conn_pool: ConnectionPool, path: &str) {
        let conn = self.conn_pool.get_connection(path);

        self.apply_migrations()

    }

    fn get_connection(&mut self, /* path */) { // <-- path for getting a connection to the correct db
        todo!()
    }

    pub fn apply_migrations(conn: RusqliteConnection) -> Result<(), ServerError> {
        Ok(migrations::update_db(&conn.inner))
    }

    pub fn get_single_value(&mut self, key: &str) -> Result<()> {
        let conn = self.conn_pool.get_connection()?;
        single_value::get(&conn, key)
    }

    pub fn set_single_value(&mut self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn_pool.get_connection()?;
        single_value::set(&conn, key, value);
    }

    pub fn execute_query(&mut self, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, Box<dyn Error>> {
        todo!()
    }

        // fn add_database(&mut self, name: &str, path: &str, db_type: DbType) -> Result<(), AddDatabaseError> {
    //     // Check if a database with the given name already exists
    //     if self.db_map.contains_key(path) {
    //         return Err(AddDatabaseError::DatabaseAlreadyExists);
    //     }
    //     todo!()
    //     // Create a new connection for the database

    //     // Add the connection to the queue and the map
    // }
}