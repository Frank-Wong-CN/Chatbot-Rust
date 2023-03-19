use rusqlite::{Connection, Result};
use crate::types::*;
use crate::utils::table_exists;

use super::schema_v1::SchemaV1 as PrevSchema;

pub struct SchemaV2;

impl SchemaV2 {
	fn check_schema_version(conn: &Connection) -> Result<u64> {
		let config_exists = table_exists(conn, "config");
		if config_exists {
			let sql = "SELECT * FROM config LIMIT 1;";
			let mut stmt = conn.prepare(sql).unwrap();
			let config: Vec<DatabaseConfig> = stmt.query_map([], |row| {
				Ok(DatabaseConfig {
					version: row.get(0)?
				})
			})
			.unwrap()
			.map(Result::unwrap)
			.collect();
			return Ok(config[0].version);
		}
		Ok(1)
	}

	fn upgrade_from_v1(conn: &Connection) -> Result<usize> {
		SchemaV2::create_schema_config(conn)?;
		SchemaV2::alter_schema_conversation(conn)?;

		Ok(0)
	}

	fn create_schema_config(conn: &Connection) -> Result<usize> {
		let sql = "
			CREATE TABLE IF NOT EXISTS config (
				version INT NOT NULL PRIMARY KEY
			);
		";
		conn.execute(sql, [])
	}

	fn alter_schema_conversation(conn: &Connection) -> Result<usize> {
		let sql = "
			ALTER TABLE conversation ADD COLUMN topic INTEGER DEFAULT 0;
		";
		conn.execute(sql, [])
	}
}

impl Schema for SchemaV2 {
	fn version() -> u64 { 2 }

	fn init_current_schema(conn: &Connection) -> Result<usize> {
		if SchemaV2::check_schema_version(conn)? != SchemaV2::version() {
			PrevSchema::init_current_schema(conn)?;
			SchemaV2::upgrade_from_v1(conn)?;
		}
		Ok(0)
	}
}