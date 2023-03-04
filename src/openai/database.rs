use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, Result};
use std::str::FromStr;

use super::types::*;

pub fn open_connection(db: String) -> Connection {
    return Connection::open(db).unwrap();
}

fn create_schema_conversation(conn: &Connection) -> Result<usize> {
    let sql = "
		CREATE TABLE IF NOT EXISTS conversation (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			title VARCHAR(512),
			key VARCHAR(512),
			updateat DATETIME DEFAULT CURRENT_TIMESTAMP
		);
	";
    conn.execute(sql, [])
}

fn create_schema_message(conn: &Connection) -> Result<usize> {
    let sql = "
		CREATE TABLE IF NOT EXISTS message (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			conversation_id INTEGER,
			role VARCHAR(32),
			content TEXT,
			prompt_tokens INTEGER,
			completion_tokens INTEGER,
			updateat DATETIME DEFAULT CURRENT_TIMESTAMP,
			FOREIGN KEY (conversation_id) REFERENCES conversation (id)
		);
	";
    conn.execute(sql, [])
}

pub fn init_schemas(conn: &Connection) -> Result<usize> {
    create_schema_conversation(conn)?;
    create_schema_message(conn)?;

    Ok(0)
}

pub fn add_conversation(conn: &Connection, title: &str, key: &str) -> Result<u32> {
    let sql = "
		INSERT INTO conversation (title, key) VALUES (?, ?);
	";
    conn.execute(sql, [title, key])?;
    return conn.query_row("SELECT last_insert_rowid();", [], |row| row.get(0));
}

pub fn get_all_conversations(conn: &Connection, key: &str) -> Result<Vec<ConversationListing>> {
    let sql = "
		SELECT
			a.id AS ID,
			a.title AS Title,
			IFNULL(SUM(b.prompt_tokens) + SUM(b.completion_tokens), 0) AS TotalUsage,
			IFNULL(MAX(b.updateat), a.updateat) AS LastUpdate
		FROM conversation a
		LEFT JOIN message b ON a.id = b.conversation_id
		WHERE a.key = ?
		GROUP BY a.id
		ORDER BY MAX(b.updateat) ASC;
	";
    let mut stmt = conn.prepare(sql).unwrap();

    let conv: Vec<ConversationListing> = stmt
        .query_map([key], |row| {
            Ok(ConversationListing {
                id: row.get(0)?,
                title: row.get(1)?,
                usage: row.get(2)?,
                lastupdate: Utc
                    .datetime_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    return Ok(conv);
}

pub fn get_all_messages_in_conversation(conn: &Connection, id: u32) -> Result<Vec<SavedMessage>> {
    let sql = "
		SELECT * FROM message WHERE conversation_id = ? ORDER BY updateat ASC;
	";
    let mut stmt = conn.prepare(sql).unwrap();

    let conv = stmt
        .query_map([id], |row| {
            Ok(SavedMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                prompt_tokens: row.get(4)?,
                completion_tokens: row.get(5)?,
                updateat: Utc
                    .datetime_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d %H:%M:%S")
                    .unwrap()
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    Ok(conv)
}

pub fn add_client_message(conn: &Connection, id: u32, msg: &str) -> Result<usize> {
	let content = msg.clone().trim().replace("\"", "\\\"");
    let sql = format!("
		INSERT INTO message (conversation_id, role, content, prompt_tokens, completion_tokens) VALUES (
			{}, ?, ?, {}, {}
		);
	", id, 0, 0);
	let mut stmt = conn.prepare(&sql)?;
    stmt.execute(["user", msg])
}

pub fn add_server_message(conn: &Connection, id: u32, msg: &CompletionResponse) -> Result<usize> {
    let role = match msg.choices[0].message.role {
        MessageRole::Assistant => "assistant",
        MessageRole::User => "user",
        MessageRole::System => "system",
    };
	let content = msg.choices[0].message.content.clone().trim().replace("\"", "\\\"");
    let sql = format!("
		INSERT INTO message (conversation_id, role, content, prompt_tokens, completion_tokens) VALUES (
			{}, ?, ?, {}, {}
		);
	", id, msg.usage.prompt_tokens, msg.usage.completion_tokens);
	let mut stmt = conn.prepare(&sql)?;
    stmt.execute([role, &content])
}
