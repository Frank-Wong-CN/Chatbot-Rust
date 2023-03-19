use rusqlite::{Connection, Result};

pub(crate) struct DatabaseConfig {
	pub version: u64
}

pub trait Schema {
	fn version() -> u64;
	fn init_current_schema(conn: &Connection) -> Result<usize>;
}
