#![allow(unused)]
use clap::Parser;
use std::{io::Write, path::PathBuf};
use spinners::{Spinner, Spinners};

mod error;
use error::MainError;
mod openai;
use openai::prelude::*;

#[derive(Debug, Parser)]
#[command(name = "ChatGPT Player")]
#[command(author = "Frank Whitefall")]
#[command(version = "1.0.0")]
#[command(about = "A terminal-based client that calls ChatGPT API to generate answers.", long_about = None)]
struct CommandLineParser {
    /// Supply API Key directly
    #[arg(short, long, value_name = "API Key")]
    key: Option<String>,

    /// Read API Key from file
    #[arg(short = 'f', long, value_name = "API Key File", default_value = "api_key")]
    key_file: Option<String>,

	// Conversation database
	#[arg(short = 'd', long, value_name = "Database", default_value = "ai.db", required = false)]
	database: String,

	// Max token
	#[arg(long, value_name = "Size", default_value = "3800")]
	max_token: usize,

	// Max remembered conversation
	#[arg(long, value_name = "Remembered Conversation", default_value = "32")]
	max_conversation: u32,
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let args = CommandLineParser::parse();
	let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();

    let mut api_key = "".into();
	
    if let Some(key) = args.key {
        api_key = key;
    }
    else if let Some(key_file) = args.key_file {
		let mut api_dir = exe_dir.clone();
		api_dir.push(key_file);
        if let Ok(str) = std::fs::read_to_string(&api_dir) {
			api_key = str;
		}
    }

    if api_key.is_empty() {
        println!("Please provide an API. See -h for more details.");
        return Ok(())
    }

	let mut db_dir = exe_dir.clone();
	db_dir.push(args.database);
	let conn = open_connection(&db_dir);
	let mut conversation_id: u32 = 0;
	let mut all_messages: Vec<SavedMessage> = vec![];
	let max_conversation_size = args.max_conversation;
	let max_conversation_token = args.max_token;
	init_schemas(&conn)?;

	let separator = String::from("===========================================================================");
    
    println!("Welcome to OpenAI Playground. Press Ctrl+C to exit the program.");

	let all_conversations = get_all_conversations(&conn, &api_key)?;
	println!("You have {} conversation(s) currently saved.", all_conversations.len());
	for conv in all_conversations.iter() {
		println!("[{}] {}: {} (Usage: {} tokens in total)", conv.lastupdate.format("%Y-%m-%d %H:%M:%S"), conv.id, conv.title, conv.usage);
	}

	println!("Enter a number to continue the desired conversation, or enter a piece of text to create a new one: ");

	loop {
		let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        prompt = prompt.trim().to_owned();

		if prompt.is_empty() {
            continue
        }

		let is_number = str::parse::<u32>(&prompt);
		if let Ok(number) = is_number {
			conversation_id = number;
			all_messages = get_all_messages_in_conversation(&conn, conversation_id)?;
		}
		else {
			conversation_id = add_conversation(&conn, &prompt, &api_key)?;
		}

		for msg in all_messages.iter() {
			println!("{}\n{}: {}", separator,
				match &msg.role[..] {
					"assistant" => "ChatGPT",
					"user" => "You",
					"system" => "System",
					_ => panic!("Database error! Message ID {} does not have a valid role!", msg.id)
				}, msg.content.trim());
		}

		break;
	}

	println!("{}", separator);

    loop {
        print!("You: ");
        std::io::stdout().flush().unwrap();

		let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        prompt = prompt.trim().to_owned();

        if prompt.is_empty() {
            continue
        }
        
        let mut spinner = Spinner::new(
            Spinners::Dots,
            "ChatGPT is thinking...".to_string(),
        );

		let mut context: Vec<Message> = vec![];
		let mut i = 0;
		let mut j = 0;
		'context_filler: for msg in all_messages.iter().rev() {
			if i > max_conversation_size || j > max_conversation_token {
				break 'context_filler;
			}
			i += 1;
			j += msg.content.len() / 3;
			let role_str = &msg.role[..];
			context.insert(0, Message {
				role: match role_str {
					"assistant" => MessageRole::Assistant,
					"user" => MessageRole::User,
					"system" => MessageRole::System,
					_ => panic!("Database error! Message ID {} does not have a valid role!", msg.id)
				},
				content: msg.content.clone()
			});
		}

		context.push(Message { role: MessageRole::User, content: prompt.clone() });

        let response = get_response(context, &api_key).await;
		match response {
			Ok(response) => {
				if !response.error() {
					add_client_message(&conn, conversation_id, &prompt);
					add_server_message(&conn, conversation_id, &response);
					all_messages = get_all_messages_in_conversation(&conn, conversation_id)?;
					spinner.stop_with_message(separator.clone());

					println!("ChatGPT: {}", response.msg().trim());
					println!("{}", separator)
				}
				else {
					spinner.stop_with_message(separator.clone());

					println!("Error: {}", response.msg().trim());
					println!("{}", separator)
				}
			},
			Err(err) => {
				spinner.stop_with_message(separator.clone());

				println!("Error: {}", err.to_string());
				println!("{}", separator)
			}
		}
    }
}
