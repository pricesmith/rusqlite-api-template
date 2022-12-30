use std::sync::{Arc, Mutex};
use std::thread;

struct ConnectionPool {
    connections: Vec<Connection>,
    next_connection: usize,
}

impl ConnectionPool {
    fn new(size: usize) -> ConnectionPool {
        let mut connections = Vec::with_capacity(size);
        for _ in 0..size {
            connections.push(Connection::new());
        }
        ConnectionPool {
            connections,
            next_connection: 0,
        }
    }

    fn get_connection(&mut self) -> Connection {
        let connection = self.connections[self.next_connection];
        self.next_connection = (self.next_connection + 1) % self.connections.len();
        connection
    }
}

struct Connection {
    // The actual connection to the database
    inner: DbConnection,
    // A flag to indicate if the connection is currently in use
    in_use: bool,
    // A reference to the database context
    context: &'static DbContext,
}

impl Connection {
    fn new() -> Connection {
        // ...
    }
}

fn main() {
    let pool = Arc::new(Mutex::new(ConnectionPool::new(10)));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let mut pool = pool.lock().unwrap();
            let connection = pool.get_connection();
            // Use the connection...
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}


// use std::thread;
// use std::sync::mpsc;

// fn main() {
//     let (tx, rx) = mpsc::channel();

//     // Spawn a new thread that will send values to the queue.
//     thread::spawn(move || {
//         tx.send(1).unwrap();
//         tx.send(2).unwrap();
//         tx.send(3).unwrap();
//     });

//     // The main thread will receive values from the queue.
//     for value in rx {
//         println!("Received value: {}", value);
//     }
// }