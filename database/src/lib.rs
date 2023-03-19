use rusqlite::Connection;
use std::path::PathBuf;

mod utils;
mod types;
pub use types::Schema;

mod versions;
pub use versions::*;

pub fn open_connection(db: &PathBuf) -> Connection {
    return Connection::open(db).unwrap();
}
