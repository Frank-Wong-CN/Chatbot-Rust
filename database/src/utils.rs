use rusqlite::{Connection, Result};
use serde_json::json;

pub fn table_exists(conn: &Connection, table_name: &str) -> bool {
    let result = conn.query_row::<i32, _, _>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        &[table_name],
        |row| row.get(0),
    );
    match result {
        Ok(count) => count > 0,
        Err(_) => false,
    }
}

#[allow(unused)]
pub fn execute_query(conn: &Connection, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<serde_json::Value> {
    let mut stmt = conn.prepare(query)?;
	let col_count = stmt.column_count();
	let mut col_names: Vec<String> = vec![];
	for col_index in 0..col_count {
		let col_name = stmt.column_name(col_index).unwrap();
		col_names.push(col_name.into());
	}
    let rows = stmt.query_map(params, |row| {
        let mut obj = json!({});
        for col_index in 0..col_count {
            let col_type = row.get::<usize, rusqlite::types::Value>(col_index)?;
            let col_val = match col_type {
                rusqlite::types::Value::Integer(int) => json!(int),
                rusqlite::types::Value::Real(float) => json!(float),
                rusqlite::types::Value::Text(string) => json!(string),
                rusqlite::types::Value::Blob(blob) => json!(blob),
                _ => json!(null),
            };
            obj[&col_names[col_index]] = col_val;
        }
        Ok(obj)
    })?;
    let mut data = Vec::new();
    for row in rows {
        data.push(row?);
    }
    Ok(json!(data))
}
