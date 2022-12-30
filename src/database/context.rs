use std::collections::HashMap;
use std::sync::{Mutex, Arc, RwLock};
use std::{fs, time::Duration};
use std::error::Error;
// use std::io::prelude::*;

use rusqlite::{Statement, Result};
use rusqlite::Connection as RusqliteConnection;

use crate::errors::ServerError;

use super::{migrations, single_value, connection::ConnectionPool};

pub enum DbType {
    REG,
    BLOB,
}

const TEST_LEVEL_0_PACKET_DB:   &str = ":memory:";
const TEST_SINGLE_VALUE_DB:     &str = ":memory";
const LEVEL_0_PACKET_DB:        &str = "level_0_packet";
const SINGLE_VALUE_DB:          &str = "single_value";

const NUM_DB: usize = 4;

fn read_sql_from_file(path: &str) -> String {
    fs::read_to_string(path).expect(r#"Failed to read SQL file."#)
}

pub struct DbContext<'a> {
    pub conn_pool: &'a ConnectionPool,
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
            conn_pool: &ConnectionPool::new(10, Duration::from_secs(30)),
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

    // fn add_database(&mut self, name: &str, path: &str, db_type: DbType) -> Result<(), AddDatabaseError> {
    //     // Check if a database with the given name already exists
    //     if self.db_map.contains_key(path) {
    //         return Err(AddDatabaseError::DatabaseAlreadyExists);
    //     }
    //     todo!()
    //     // Create a new connection for the database

    //     // Add the connection to the queue and the map
    // }

    fn get_connection() {}

    fn create_connection_pool(&mut self, db_file: &str) -> Result<Arc<Mutex<RusqliteConnection>>, Box<dyn Error>> {
        let conn = self.conn_pool.get_connection(db_file)?; 
    }

    pub fn apply_migrations(&mut self) -> Result<(), ServerError> {
        let conn = self.conn_pool.get_connection()?;
        Ok(migrations::update_db(&conn.inner))
    }

    pub fn get_single_value(&mut self, key: &str) -> Result<()> {
        let conn = self.conn_pool.get_connection()?;
        single_value::get(&conn, key)
    }

    pub fn set_single_value(&mut self, key: &str, value: &str) -> Result<()> {
        let conn = self.pool.get_connection()?;
        single_value::set(&conn, key, value);
    }

    pub fn execute_query(&mut self, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, Box<dyn Error>> {
        todo!()
    }
}