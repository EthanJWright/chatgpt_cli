use chatgpt::prelude::*;
use chatgpt::types::CompletionResponse;
use std::env::args;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

const CONVERSATIONS_DIR: &str = "conversations";
const CONVERSATION: &str = "conversations/conversation.json";

fn conversation_file_path(name: &str) -> PathBuf {
    PathBuf::from(CONVERSATIONS_DIR).join(format!("conversation_{}.json", name))
}

#[tokio::main]
async fn main() -> chatgpt::Result<()> {

    // Get the API key from the command line
    let key = args().nth(1).unwrap();


    let client = ChatGPT::new(&key)?;

    std::fs::create_dir_all(CONVERSATIONS_DIR)?;

    // Skip the first argument (the program name) and collect the rest into a Vec<String>
    let args_vec: Vec<String> = args().skip(2).collect();


    // Join the collected arguments into a single sentence
    // only if there are more than 1 arguments
    let message = if args_vec.len() > 0 {
        Some(args_vec.join(" "))
    } else {
        None
    };

    let input: String = if let Some(message) = message {
        message
    } else {
        println!("Enter your command: ");
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        input
    };

    // remove the first command from the arguments
    let first_command = input.trim().split_whitespace().next().unwrap();
    let args_vec: Vec<String> = args_vec.into_iter().skip(1).collect();

    match first_command.trim() {
        "flush" => flush_conversation(),
        "save" => save_conversation(&client, &args_vec).await,
        "load" => load_conversation(&client, &args_vec).await,
        "clear" => clear_conversations(),
        "list" => {
            println!("Saved conversations:");
            print_saved_conversations();
            Ok(())
        }
        _ => process_message(&client, input.trim()).await,
    }
}

async fn save_conversation(client: &ChatGPT, args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Saving conversation as {}", name);
        conversation_file_path(name)
    } else {
        println!("What should I save it as?");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        conversation_file_path(input.trim())
    };

    client
        .restore_conversation_json(CONVERSATION)
        .await?
        .save_history_json(file_name)
        .await?;

    std::fs::remove_file(CONVERSATION)?;

    Ok(())
}

async fn load_conversation(client: &ChatGPT, args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Loading conversation from {}", name);
        conversation_file_path(name)
    } else {
        println!("What should I load it from?");
        stdout().flush()?;
        print_saved_conversations();
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        conversation_file_path(input.trim())
    };

    let conversation: Conversation = client.restore_conversation_json(file_name).await?;
    conversation.save_history_json(CONVERSATION).await?;

    Ok(())
}

fn flush_conversation() -> chatgpt::Result<()> {
    std::fs::remove_file(CONVERSATION)?;

    Ok(())
}

fn clear_conversations() -> chatgpt::Result<()> {
    let mut conversations = std::fs::read_dir(CONVERSATIONS_DIR)?;

    while let Some(conversation) = conversations.next() {
        let conversation = conversation?;

        if is_saved_conversation(&conversation) {
            let print_name = conversation.file_name().into_string().unwrap();
            println!("Removing - {}", print_name.replace("conversation_", "").replace(".json", ""));
            std::fs::remove_file(conversation.path())?;
        }
    }

    Ok(())
}

async fn process_message(client: &ChatGPT, message: &str) -> chatgpt::Result<()> {
    let mut conversation: Conversation = if std::path::Path::new(CONVERSATION).exists() {
        client.restore_conversation_json(CONVERSATION).await?
    } else {
        client.new_conversation()
    };

    let response: CompletionResponse = conversation.send_message(message.to_string()).await?;
    println!("{}", response.message().content);
    conversation.save_history_json(CONVERSATION).await?;

    Ok(())
}

fn print_saved_conversations() {
    let conversations = std::fs::read_dir(CONVERSATIONS_DIR).unwrap();
    for conversation in conversations {
    if let Ok(conversation) = conversation {
        if is_saved_conversation(&conversation) {
            let print_name = conversation.file_name().into_string().unwrap();
            println!("{}", print_name.replace("conversation_", "").replace(".json", ""));
        }
    }
}
}

fn is_saved_conversation(conversation: &std::fs::DirEntry) -> bool {
    let file_name = conversation.file_name().into_string().unwrap();
    file_name.starts_with("conversation_") && file_name.ends_with(".json")
}

