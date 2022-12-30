use std::collections::HashMap;
use std::error::Error;
use std::{path, ptr};
// use std::path::PathBuf;
// use std::path::Path;
use std::time::Duration;
use std::sync::{Arc, Mutex};
// use log::info;
// use rusqlite::Connection;
use rusqlite::{Connection as RusqliteConnection};

use super::context::DbContext;

// use super::{sqlite, context::DbContext};

pub struct ConnectionPools {
    pool_map: HashMap<String, Arc<Mutex<ConnectionPool>>>,
}

impl ConnectionPools {
    pub fn new() -> Self {
        ConnectionPools {
            pool_map: HashMap::new(),
        }
    }

    pub fn get_connection(&self, db_name: &str) -> Result<Connection, ConnectionPoolError> {
        let pool = self.pool_map.get(db_name).ok_or(ConnectionPoolError::PoolNotFound)?;
        pool.lock().unwrap().get_connection()
    }

    pub fn add_pool(&mut self, name: &str, pool: ConnectionPool) {
        let arc_pool = Arc::new(Mutex::new(pool));
        self.pool_map.insert(name.to_owned(), arc_pool);
    }
}

pub struct ConnectionPool {

    /// The maximum number of connections allowed in the pool.
    max_size: usize,

    /// The timeout for waiting for a connection to become available.
    timeout: Duration,

    /// The file paths or in-memory connection for the connections in the pool.
    // file_paths: Vec<PathBuf>,
    // in_memory: Option<Arc<Mutex<Connection>>>,

    /// The list of available connections.
    available_connections: Vec<Arc<Mutex<Connection<'static>>>>,

    /// The list of connections currently in use.
    in_use_connections: Vec<Arc<Mutex<Connection<'static>>>>,
}

/// A connection pool for managing connections to the database.
impl ConnectionPool {
    /// Creates a new connection pool with the given maximum size and timeout.
    pub fn new(max_size: usize, timeout: Duration) -> Self {
        ConnectionPool {
            max_size,
            timeout,
            available_connections: Vec::with_capacity(max_size),
            in_use_connections: Vec::with_capacity(max_size),
        }
    }

    /// Gets a connection from the pool. If no connection is available, this function will block
    /// until one becomes available or until the timeout has been reached.
    pub fn get_connection(&self) -> Result<Connection, ConnectionPoolError> {
        let start_time = std::time::Instant::now();

        loop {
            // Try to get a connection from the available connections list.
            if let Some(conn) = self.available_connections.pop() {
                self.in_use_connections.push(conn.clone());
                return Ok(*conn.lock().unwrap());
            }

            // If the available connections list is empty and the pool is not at capacity, create
            // a new connection and return it.
            if self.in_use_connections.len() < self.max_size {
                let conn = Arc::new(Mutex::new(Connection::open(":memory:", &self)?));
                self.in_use_connections.push(conn.clone());
                return Ok(*conn.lock().unwrap());
            }

            // If the timeout has been reached, return an error.
            if start_time.elapsed() >= self.timeout {
                return Err(ConnectionPoolError::Timeout);
            }

            // Sleep for a short period of time before trying again.
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Returns a reference to the last connection in the available_connections list, 
    /// or None if the list is empty.
    pub fn borrow_connection(&self) -> Option<&Arc<Mutex<Connection>>> {
        self.available_connections.last()
    }

    pub fn return_connection(&mut self, conn: &Arc<Mutex<Connection>>) {
        // Search for the connection in the in_use_connections list and remove it if it's found.
        if let Some(pos) = self.in_use_connections.iter().position(|c| Arc::ptr_eq(c, conn)) {
            self.in_use_connections.remove(pos);
        }

        // Add the connection to the available_connections list.
        self.available_connections.push(conn.clone());
    }

    /// Searches for the connection in the available_connections list and remove it if it's found.
    pub fn remove_connection(&mut self, conn: &Arc<Mutex<Connection>>) -> Option<Arc<Mutex<Connection>>> {
        let index = self.available_connections.iter().position(|c| Arc::ptr_eq(c, conn))?;
        Some(self.available_connections.remove(index))
    }

    pub fn drop_connection(&mut self, conn: &Connection) {
        // Search for the connection in the in_use_connections list and remove it if it's found.
        let pos = self.in_use_connections.iter().position(|c| *c.lock().unwrap() == *conn).expect("Connection not found in in_use_connections list");
        self.in_use_connections.remove(pos);
    
        // Add the connection to the available_connections list.
        self.available_connections.push(conn.clone());
    }
}

#[derive(Debug)]
pub enum ConnectionPoolError {

    /// An error occurred while trying to create a new connection.
    ConnectionError(Box<dyn Error>),

    /// The timeout was reached while waiting for a connection to become available.
    Timeout,

    /// The requested pool was not found.
    PoolNotFound,
}

impl std::fmt::Display for ConnectionPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionPoolError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            ConnectionPoolError::Timeout => write!(f, "Timed out while waiting for a connection"),
            ConnectionPoolError::PoolNotFound => write!(f, "Requested pool was not found"),
        }
    }
}

impl Error for ConnectionPoolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConnectionPoolError::ConnectionError(e) => Some(e.as_ref()),
            ConnectionPoolError::Timeout => None,
            ConnectionPoolError::PoolNotFound => None,
        }
    }
}

// ?: Possibly abstract to switch db's later? Maybe I'm too bored. 
// pub struct Connection(Arc<Mutex<RusqliteConnection>>);

struct Connection<'a> {
    // The actual connection to the database
    inner: RusqliteConnection,
    // A flag to indicate if the connection is currently in use
    in_use: bool,
    // A reference to the connection pool
    pool: &'a ConnectionPool,
}

impl<'a> Connection<'a> {
    pub fn open(path: &str, pool: &'a ConnectionPool) -> Result<Self, Box<dyn Error>> {
        let inner = RusqliteConnection::open(path)?;
        Ok(Connection { inner, in_use: true, pool })
    }
}

impl<'a> PartialEq for Connection<'a> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(&self.inner, &other.inner)
    }
}

impl<'a> Drop for Connection<'a> {
    fn drop(&mut self) {
        self.in_use = false;
        let conn = Arc::new(Mutex::new(std::mem::replace(self, Connection::default())));
        self.pool.return_connection(&conn);
    }
}

// pub fn open_or_create_from_file(dbfile: &Path) -> Result<Connection, Box<dyn Error>> {
//     let conn = Connection::open(dbfile)?;

//     // conn.busy_timeout(Duration::from_millis(500))?;
//     conn.busy_handler(Some(|count| {
//         info!("busy_handler: {}", count);

//         let d = Duration::from_millis(500);
//         std::thread::sleep(d);

//         true
//     }))?;

//     conn.execute("PRAGMA foreign_keys = true;", params![])?;

//     Ok(conn)
// }

// pub fn open_in_memory() -> Result<()> {
//     let conn = sqlite::open_in_memory();

//     conn.execute("PRAGMA foreign_keys = true;", params![])?;

//     let db = DbContext::new(&conn);

//     Ok(db)
// }