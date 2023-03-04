#![allow(unused)]
use reqwest;
use clap::Parser;
use std::io::Write;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};


#[derive(Debug, Parser)]
#[command(name = "ChatGPT Player")]
#[command(author = "Frank Whitefall")]
#[command(version = "1.0.0")]
#[command(about = "A terminal-based client that calls ChatGPT API to generate answers.", long_about = None)]
struct CommandLineParser {
    /// Supply API Key directly
    #[arg(short, long, value_name = "API KEY")]
    key: Option<String>,

    /// Read API Key from file
    #[arg(short = 'f', long, value_name = "API KEY FILE")]
    key_file: Option<String>
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
	messages: Vec<Message>
}

#[derive(Deserialize)]
struct CompletionResponse {
	id: String,
	object: String,
	created: u64,
	model: String,
	usage: TokenUsage,
    choices: Vec<ResponseChoice>,
}

#[derive(Deserialize)]
struct TokenUsage {
	prompt_tokens: u32,
	completion_tokens: u32,
	total_tokens: u64
}

#[derive(Deserialize)]
struct ResponseChoice {
	index: u64,
	finish_reason: String,
    message: Message
}

#[derive(Serialize, Deserialize)]
struct Message {
	role: MessageRole,
	content: String
}

#[derive(Deserialize)]
enum MessageRole {
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

async fn get_response(prompt: &str, api_key: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let request = CompletionRequest {
        model: "gpt-3.5-turbo".into(),
		messages: vec![Message { role: MessageRole::User, content: prompt.to_string() }]
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await;

	return match response {
		Ok(success) => {
			Ok(success.json::<CompletionResponse>().await?.choices[0].message.content.clone())
		},
		Err(failed) => {
			Ok(failed.to_string())
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = CommandLineParser::parse();

    let mut api_key = "".into();
    if let Some(key_file) = args.key_file {
        api_key = std::fs::read_to_string(key_file)?;
    }
    if let Some(key) = args.key {
        api_key = key;
    }
    if api_key.is_empty() {
        println!("Please provide an API. See -h for more details.");
        return Ok(())
    }
    
    println!("Welcome to OpenAI Playground. Press Ctrl+C to exit the program.");

    loop {
        print!("You: ");
        std::io::stdout().flush().unwrap();

		let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        prompt = prompt.trim().to_owned();

        if prompt.is_empty() {
            // println!("Prompt empty!");
            continue
        }
        
        let mut spinner = Spinner::new(
            Spinners::Dots,
            "ChatGPT is thinking...".to_string(),
        );

        let response = get_response(&prompt, &api_key[..]).await.unwrap();

        spinner.stop_with_message("===========================================================================".into());

        println!("ChatGPT: {}", response.trim());
		println!("===========================================================================")
    }
}
