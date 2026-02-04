// Connection pool

pub struct ConnectionPool {
    // Placeholder for connection pooling
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self { max_connections }
    }

    pub fn max_connections(&self) -> usize {
        self.max_connections
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(10)
    }
}
