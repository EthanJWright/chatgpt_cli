use chatgpt::prelude::*;
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
    let key = std::env::args().nth(1).expect("API key not provided");
    let config = ModelConfigurationBuilder::default()
        .engine(ChatGPTEngine::Gpt35Turbo)
        .build()
        .unwrap();

    let client = match ChatGPT::new_with_config(&key, config) {
        Ok(val) => val,
        Err(err) => panic!("Failed to create ChatGPT client: {}", err),
    };


    std::fs::create_dir_all(CONVERSATIONS_DIR)?;

    // Skip the first two arguments (the executable and the API key)
    let args_vec: Vec<String> = args().skip(2).collect();

    // If there are any arguments, use them as the message
    let message = if args_vec.len() > 0 {
        Some(args_vec.join(" "))
    } else {
        None
    };

    // If there is no message, prompt the user for one
    let input: String = if let Some(message) = message {
        message
    } else {
        println!("Enter your command: ");
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        input
    };

    let first_command = input.trim().split_whitespace().next().unwrap();
    // remove the potential command from the arguments
    // in process_message we will just use the raw input
    let args_vec: Vec<String> = args_vec.into_iter().skip(1).collect();

    match first_command.trim() {
        "flush" => flush_conversation(),
        "save" => save_conversation(&client, &args_vec).await,
        "remove" => remove_conversation(&args_vec).await,
        "load" => load_conversation(&client, &args_vec).await,
        "clear" => clear_conversations(),
        "list" => {
            println!("Saved conversations:");
            print_saved_conversations();
            Ok(())
        }
        _ => {
            let saved = get_saved_conversations();
            if saved.contains(&input.trim().to_string()) {
                load_conversation(&client, &[input.trim().to_string()]).await
            } else {
                process_message(&client, input.trim()).await
            }
        }
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

    Ok(())
}

async fn remove_conversation(args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Removing conversation {}", name);
        conversation_file_path(name)
    } else {
        println!("What should I remove?");
        print_saved_conversations();
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        conversation_file_path(input.trim())
    };

    std::fs::remove_file(file_name)?;

    Ok(())
}

async fn load_conversation(client: &ChatGPT, args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Loading conversation from {}", name);
        conversation_file_path(name)
    } else {
        println!("What should I load it from?");
        print_saved_conversations();
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        conversation_file_path(input.trim())
    };

    let conversation = client.restore_conversation_json(file_name).await?;
    conversation.save_history_json(CONVERSATION).await?;

    Ok(())
}

fn flush_conversation() -> chatgpt::Result<()> {
    std::fs::remove_file(CONVERSATION)?;
    Ok(())
}

fn clear_conversations() -> chatgpt::Result<()> {
    // Add a confirmation prompt
    println!("Are you sure you want to delete all saved conversations? (y/n)");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    if input.trim() != "y" {
        return Ok(());
    }

    let conversations = std::fs::read_dir(CONVERSATIONS_DIR)?;
    for conversation in conversations {
        let conversation = conversation?;
        if is_saved_conversation(&conversation) {
            let print_name = conversation.file_name().into_string().unwrap();
            println!("Removing - {}", print_name.trim_start_matches("conversation_").trim_end_matches(".json"));
            std::fs::remove_file(conversation.path())?;
        }
    }

    Ok(())
}

async fn process_message(client: &ChatGPT, message: &str) -> chatgpt::Result<()> {
    let mut conversation = if std::path::Path::new(CONVERSATION).exists() {
        client.restore_conversation_json(CONVERSATION).await?
    } else {
        client.new_conversation()
    };

    let response = conversation.send_message(message.to_string()).await?;
    
    // Print two new lines to separate the conversation
    println!("\n\n{}", response.message().content);
    
    conversation.save_history_json(CONVERSATION).await?;
    
    Ok(())
}

fn get_saved_conversations() -> Vec<String> {
    let conversations = std::fs::read_dir(CONVERSATIONS_DIR).unwrap();
    let mut names: Vec<String> = Vec::new();

    for conversation in conversations {
        if let Ok(conversation) = conversation {
            if is_saved_conversation(&conversation) {
                let print_name = conversation.file_name().into_string().unwrap();
                names.push(print_name.trim_start_matches("conversation_").trim_end_matches(".json").to_string());
            }
        }
    }

    names
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

