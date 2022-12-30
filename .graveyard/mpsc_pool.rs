use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use std::collections::VecDeque;

use rusqlite::Connection as RusqliteConnection;
use mpsc::{sync_channel, Receiver, SyncSender};

struct ConnectionPool {
    /// The maximum number of connections allowed in the pool.
    max_size: usize,
    /// The timeout for waiting for a connection to become available.
    timeout: Duration,
    /// The sender and receiver for the queue of connections.
    available_connections: SyncSender<Arc<RwLock<<RusqliteConnection>>>,
    in_use_connections: Receiver<Arc<RwLock<<RusqliteConnection>>>,
}

impl ConnectionPool {
    pub fn new(max_size: usize, timeout: Duration) -> Self {
        // Create the sender and receiver for the queue of connections.
        let (available_connections, in_use_connections) = mpsc::channel();

        // Create and populate the queue with connections.
        for _ in 0..max_size {
            sender.send(RusqliteConnection::open(":memory:").unwrap()).unwrap();
        }

        ConnectionPool { max_size, timeout, available_connections, in_use_connections }
    }

    pub fn get_connection(&self) -> Result<RusqliteConnection, String> {
        // Wait for a connection to become available or until the timeout has been reached.
        let start_time = std::time::Instant::now();
        loop {
            if let Ok(conn) = self.receiver.try_recv() {
                return Ok(conn);
            }

            if start_time.elapsed() >= self.timeout {
                return Err("Timed out waiting for a connection".to_owned());
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

}