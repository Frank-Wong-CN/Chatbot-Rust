use std::string::ToString;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestError {
    status: Option<String>,
    url: Option<String>,
    error: String,
}

impl RequestError {
	pub fn new(err: reqwest::Error) -> Self {
		RequestError {
			status: err.status().map(|s| s.to_string()),
			url: err.url().map(|u| u.to_string()),
			error: err.to_string(),
		}
	}
}

impl ToString for RequestError {
	fn to_string(&self) -> String {
		return serde_json::to_string(&self).unwrap();
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONParseError {
    error: String,
	original: String
}

impl JSONParseError {
	pub fn new(err: String, text: String) -> Self {
		JSONParseError { error: err, original: text }
	}
}

impl ToString for JSONParseError {
	fn to_string(&self) -> String {
		return serde_json::to_string(&self).unwrap();
	}
}