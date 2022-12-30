use std::sync::RwLock;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_utils::sync::{SeqQueue, PushError, PopError};
use rusqlite::Connection as RusqliteConnection;

use super::context::DbType;

pub struct ConnectionPool {
    max_size: usize,
    timeout: Duration,
    // idle_timeout: Duration,
    last_used_time: SystemTime,
    queue: SeqQueue<(Arc<RwLock<RusqliteConnection>>, DbType, AtomicBool, SystemTime)>,
}

impl ConnectionPool {
    pub fn new(max_size: usize, timeout: Duration, /* idle_timeout: Duration */) -> Self {
        ConnectionPool {
            max_size,
            timeout,
            // idle_timeout,
            last_used_time: SystemTime::now(),
            queue: SeqQueue::new(),
        }
    }

    pub fn get_connection(&self, path: &str, db_type: DbType) -> Result<(Arc<RwLock<RusqliteConnection>>, DbType, AtomicBool, SystemTime), PopError>  {
        loop {
            // Try to pop a connection from the queue
            if let Some((conn, in_use)) = self.queue.pop() {
                // If the connection is not in use, return it
                if !in_use.load(Ordering::SeqCst) {
                    in_use.store(true, Ordering::SeqCst);
                    // Update connection and connection pool last used times
                    conn.last_used_time = SystemTime::now();
                    self.last_used_time = SystemTime::now();
                    return Ok(conn);
                } else {
                    // If the connection is in use, push it back to the queue and try again
                    self.queue.push((conn, in_use)).unwrap();
                }
            } else {
                // If the queue is empty, create a new connection if we haven't reached the max size
                if self.queue.len() < self.max_size {
                    let conn = Arc::new(RwLock::new(RusqliteConnection::open(path)?));
                    let in_use = AtomicBool::new(true);
                    return Ok((conn, in_use, SystemTime::now()));
                } else {
                    // If the queue is empty and we've reached the max size, return an error
                    return Err(PopError);
                }
            }
        }
    }

    pub fn return_connection(&self, conn: Arc<RwLock<RusqliteConnection>>) -> Result<(), ConnectionPoolError> {
        // Find the connection in the queue and set its in_use flag to false
        let mut found = false;
        for (c, in_use) in self.queue.iter_mut() {
            if Arc::ptr_eq(c, &conn) {
                in_use.store(false, Ordering::SeqCst);
                found = true;
                break;
            }
        }
        if !found {
            return Err(ConnectionPoolError::ConnectionNotFound);
        }
        // Push the connection back to the queue
        match self.queue.push((conn, AtomicBool::new(false))) {
            Ok(_) => Ok(()),
            Err(PushError) => Err(ConnectionPoolError::QueueFull),
        }
    }

    pub fn drop_connection(&self, conn: &Arc<RwLock<RusqliteConnection>>) -> Result<(), PopError> {
        // Iterate over the queue
        for i in 0..self.queue.len() {
            // Get the current connection from the queue
            let (current_conn, in_use) = self.queue.get(i).unwrap();

            // Compare the current connection with the one we want to remove
            if Arc::ptr_eq(current_conn, conn) {
                // If they are the same, close the connection --
                conn.write().unwrap().close().unwrap();
                // -- and remove it from the queue
                self.queue.remove(i);
                return Ok(());
            }
        }
        // If we reach here, the connection was not found in the queue
        Err(PopError)
    }
}

#[derive(Debug)]
enum ConnectionPoolError {
    ConnectionNotFound,
    QueueFull,
}

#[test]
fn test_get_connection() {
    // Set up a connection pool with a maximum size of 3 and a timeout of 5 seconds
    let pool = ConnectionPool::new(3, Duration::from_secs(5));

    // Try to get a connection from the pool
    let conn = pool.get_connection(":memory:");
    assert!(conn.is_ok());
    let conn = conn.unwrap();

    // Check that the connection is valid
    assert!(conn.read().unwrap().is_open());
}

#[test]
fn test_return_connection() {
    // Set up a connection pool with a maximum size of 3 and a timeout of 5 seconds
    let pool = ConnectionPool::new(3, Duration::from_secs(5));

    // Get a connection from the pool
    let conn = pool.get_connection(":memory:").unwrap();

    // Return the connection to the pool
    assert!(pool.return_connection(conn).is_ok());
}

#[test]
fn test_drop_connection() {
    // Set up a connection pool with a maximum size of 3 and a timeout of 5 seconds
    let pool = ConnectionPool::new(3, Duration::from_secs(5));

    // Get a connection from the pool
    let conn = pool.get_connection(":memory:").unwrap();

    // Drop the connection from the pool
    assert!(pool.drop_connection(&conn).is_ok());

    // Try to get the connection again
    let conn = pool.get_connection(":memory:");
    assert!(conn.is_err());
}

#[test]
fn test_get_connection_when_queue_is_full() {
    // Create a new connection pool with a maximum size of 1
    let pool = ConnectionPool::new(1, Duration::from_secs(3600));
    // Try to get a connection from the pool
    assert!(pool.get_connection(":memory:").is_ok());
    // Try to get another connection from the pool, should fail since the queue is full
    assert!(pool.get_connection(":memory:").is_err());
}

#[test]
fn test_get_connection_and_timeout() {
    // Create a new connection pool with a timeout of 100 milliseconds
    let pool = ConnectionPool::new(10, Duration::from_millis(100));
    // Try to get a connection from the pool
    assert!(pool.get_connection(":memory:").is_ok());
    // Wait for 200 milliseconds
    std::thread::sleep(Duration::from_millis(200));
    // Try to get another connection from the pool, should fail since the timeout has been reached
    assert!(pool.get_connection(":memory:").is_err());
}
