use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct ConversationListing {
	pub id: u32,
	pub title: String,
	pub usage: u64,
	pub lastupdate: DateTime<Utc>
}

pub struct SavedMessage {
	pub id: u32,
	pub conversation_id: u32,
	pub role: String,
	pub content: String,
	pub prompt_tokens: u64,
	pub completion_tokens: u64,
	pub updateat: DateTime<Utc>
}

#[derive(Serialize, Deserialize)]
pub enum OpenAIResponse {
	Success(CompletionResponse),
	Failure(OpenAIError)
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIError {
	pub error: CompletionError
}

#[derive(Serialize, Deserialize)]
pub struct CompletionError {
	pub message: String,
	pub r#type: String,
	pub param: Option<String>,
	pub code: String
}

#[derive(Serialize)]
pub struct CompletionRequest {
    pub model: String,
	pub messages: Vec<Message>
}

#[derive(Serialize, Deserialize)]
pub struct CompletionResponse {
	pub id: String,
	pub object: String,
	pub created: u64,
	pub model: String,
	pub usage: TokenUsage,
    pub choices: Vec<ResponseChoice>,
}

impl CompletionResponse {
	pub fn msg(&self) -> String {
		return self.choices[0].message.content.clone();
	}
}

#[derive(Serialize, Deserialize)]
pub struct TokenUsage {
	pub prompt_tokens: u64,
	pub completion_tokens: u64,
	pub total_tokens: u64
}

#[derive(Serialize, Deserialize)]
pub struct ResponseChoice {
	pub index: u64,
	pub finish_reason: Option<String>,
    pub message: Message
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
	pub role: MessageRole,
	pub content: String
}

#[derive(Deserialize, Clone)]
pub enum MessageRole {
	#[serde(rename = "assistant")]
	Assistant,
	
	#[serde(rename = "user")]
	User,
	
	#[serde(rename = "system")]
	System
}

impl Serialize for MessageRole {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer {
		match *self {
			Self::Assistant => serializer.serialize_str("assistant"),
			Self::User => serializer.serialize_str("user"),
			Self::System => serializer.serialize_str("system")
		}
	}
}
