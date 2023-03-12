use clap::Parser;
use serde_json::json;
use openai::prelude::*;
use std::{io::Write, path::PathBuf};
use spinners::{Spinner, Spinners};

mod error;
use error::*;
mod types;
use types::*;

static SEPARATOR: &str = "===========================================================================";

#[derive(Debug, Parser)]
#[command(name = "ChatGPT Player")]
#[command(author = "Frank Whitefall")]
#[command(version = "1.1.0")]
#[command(about = "A terminal-based client that calls ChatGPT API to generate answers.", long_about = None)]
struct CommandLineParser {
    /// Supply API Key directly
    #[arg(short, long, value_name = "API Key")]
    key: Option<String>,

    /// Read API Key from file
    #[arg(short = 'f', long, value_name = "API Key File", default_value = "$api_key")]
    key_file: String,

	// Conversation database
	#[arg(short = 'd', long, value_name = "Database", default_value = "$ai.db")]
	database: String,

	// Max token
	#[arg(long, value_name = "Size", default_value = "3800")]
	max_token: u64,

	// Max remembered conversation
	#[arg(long, value_name = "Remembered Conversation", default_value = "32")]
	max_dialog: u64,

	// Proxy
	#[arg(short, long, value_name = "Proxy Address, for example: \"socks5://127.0.0.1:1080\"")]
	proxy: Option<String>,
}

fn init() -> Result<ChatManager, ArgumentError> {
	let args = CommandLineParser::parse();
	let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
	let cw_dir = std::env::current_dir().unwrap().to_path_buf();

    let mut api_key = String::new();
	
    if let Some(key) = args.key {
        api_key = key;
    }
    else {
		let mut api_dir: PathBuf;
		if args.key_file.starts_with("$") {
			api_dir = exe_dir.clone();
			api_dir.push(&args.key_file[1..]);
		}
		else {
			api_dir = cw_dir.clone();
			api_dir.push(args.key_file);
		}

		if let Ok(str) = std::fs::read_to_string(&api_dir) {
			api_key = str;
		}
    }

    if api_key.is_empty() {
		let error = ArgumentError::new("api_key", "No API Key!");
        return Err(error);
    }

	let mut db_dir: PathBuf;
	if args.database.starts_with("$") {
		db_dir = exe_dir.clone();
		db_dir.push(&args.database[1..]);
	}
	else {
		db_dir = cw_dir.clone();
		db_dir.push(args.database);
	}

	let conn = open_connection(&db_dir);
	let max_dialog = args.max_dialog;
	let max_token = args.max_token;
	let proxy = args.proxy;

	Ok(ChatManager {
		max_token,
		max_dialog,
		api_key,
		proxy,
		connection: conn,
		current_session: None
	})
}

fn create_session(mgr: &ChatManager) -> Result<ChatSession, MainError> {
	let conversation_id: u32;
	let mut all_conv_id: Vec<u32> = vec![];
	let mut all_messages: Vec<SavedMessage> = vec![];
	init_schemas(&mgr.connection)?;

	let all_conversations = get_all_conversations(&mgr.connection, &mgr.api_key)?;
	println!("You have {} conversation(s) currently saved.", all_conversations.len());
	for conv in all_conversations.iter() {
		all_conv_id.push(conv.id);
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
			if !all_conv_id.contains(&number) {
				println!("No such conversation. Please enter again: ");
				continue
			}
			conversation_id = number;
			all_messages = get_all_messages_in_conversation(&mgr.connection, conversation_id)?;
		}
		else {
			conversation_id = add_conversation(&mgr.connection, &prompt, &mgr.api_key)?;
		}

		for msg in all_messages.iter() {
			println!("{}\n{}: {}", SEPARATOR,
				match &msg.role[..] {
					"assistant" => "ChatGPT",
					"user" => "You",
					"system" => "System",
					_ => panic!("Database error! Message ID {} does not have a valid role!", msg.id)
				}, msg.content.trim());
		}

		break;
	}

	Ok(ChatSession { conversation_id, history: all_messages, prompt: String::new() })
}

async fn execute_chat(mgr: &mut ChatManager) -> Result<(), MainError> {
	let mut session = mgr.current_session.as_mut().unwrap();
	let mut spinner = Spinner::new(
		Spinners::Dots,
		"ChatGPT is thinking...".to_string(),
	);

	let mut context: Vec<Message> = vec![];
	let mut i = 0;
	let mut j = 0;
	'context_filler: for msg in session.history.iter().rev() {
		if i > mgr.max_dialog || j > mgr.max_token {
			break 'context_filler;
		}
		i += 1;
		j += msg.content.len() as u64 / 3;
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

	context.push(Message { role: MessageRole::User, content: session.prompt.clone() });

	let openai_response = get_response(&context, &mgr.api_key, &mgr.proxy).await;
	match openai_response {
		Ok(response) => match response {
			OpenAIResponse::Success(completion_response) => {
				add_client_message(&mgr.connection, session.conversation_id, &session.prompt)?;
				add_server_message(&mgr.connection, session.conversation_id, &completion_response)?;
				session.history = get_all_messages_in_conversation(&mgr.connection, session.conversation_id)?;
				spinner.stop_with_message(SEPARATOR.into());

				println!("ChatGPT: {}", completion_response.msg().trim());
			},
			OpenAIResponse::Failure(openai_error) => {
				add_error_log(&mgr.connection, &mgr.api_key, &context, &json!(openai_error).to_string(), Some(&openai_error))?;
				spinner.stop_with_message(SEPARATOR.into());

				println!("Error: {}", openai_error.error.message);
			}
		},
		Err(err) => {
			add_error_log(&mgr.connection, &mgr.api_key, &context, &err, None)?;
			spinner.stop_with_message(SEPARATOR.into());

			println!("Error: {}", err);
		}
	}

	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
	let mut mgr: ChatManager;
	match init() {
		Ok(manager) => mgr = manager,
		Err(error) => match error.argument.as_str() {
			"api_key" => {
				println!("Please provide an API Key. See -h for more details.");
				std::process::exit(1);
			},
			_ => {
				panic!("{}", error);
			}
		}
	};
	
    println!("Welcome to OpenAI Playground. Press Ctrl+C to exit the program.");

	while mgr.current_session.is_none() {
		match create_session(&mgr) {
			Ok(session) => mgr.current_session = Some(session),
			Err(error) => {
				panic!("{}", error);
			}
		}
	}

	println!("{}", SEPARATOR);

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

		let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        prompt = prompt.trim().to_owned();

		mgr.current_session.as_mut().unwrap().prompt = prompt.clone();

        if prompt.is_empty() {
            continue
        }
		else if prompt.starts_with("/") {
			println!("This is a command: {}. Custom commands are not implemented yet.", &prompt[1..]);
		}
        else if let Err(error) = execute_chat(&mut mgr).await {
			panic!("{}", error)
		}

		println!("{}", SEPARATOR);
    }
}
