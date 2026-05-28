// Database module — stubs, will be implemented in later tasks

pub mod connection {
    use std::path::Path;

    pub fn open(path: &Path) -> Result<rusqlite::Connection, rusqlite::Error> {
        todo!("db connection open")
    }
}

pub mod migrate {
    use rusqlite::Connection;

    pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
        todo!("db migrate run")
    }
}
