use openai::types::SavedMessage;
use rusqlite::Connection;

pub struct ChatManager {
	pub max_token: u64,
	pub max_dialog: u64,
	pub api_key: String,
	pub connection: Connection,
	pub proxy: Option<String>,
	pub current_session: Option<ChatSession>
}

pub struct ChatSession {
	pub conversation_id: u32,
	pub history: Vec<SavedMessage>,
	pub prompt: String
}