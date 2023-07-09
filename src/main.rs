use chatgpt::prelude::*;
extern crate shellexpand;
use std::fs::canonicalize;
use std::fs::File;
use std::env::args;
use std::io::{stdin, stdout, Write};
use std::io::{BufRead, BufReader};

mod ai;
mod file;
mod client;

const CHUNK_SIZE: usize = 20000;
const CHUNK_BATCH_SIZE: usize = 1;


#[tokio::main]
async fn main() -> chatgpt::Result<()> {
    // Get the API key from the command line
    let key = std::env::args().nth(1).expect("API key not provided");
    let client_key = key.clone();
    let client = client::get_client(client_key).await;

    std::fs::create_dir_all(file::conversations_dir().unwrap())?;

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
        "help" => {
            println!("Commands:");
            println!("  help: print this help message");
            println!("  flush: clear the current conversation");
            println!("  save [name]: save the current conversation");
            println!("  remove [name]: remove a saved conversation");
            println!("  load [name]: load a saved conversation");
            println!("  clear: clear all saved conversations");
            println!("  list: list all saved conversations");
            println!(
                "  --file=[file] [message]: send a message related to a file that is also uploaded"
            );
            println!("  [message]: send a message");
            Ok(())
        }
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
            if input.contains("--file=") {
                return message_with_file(key, &[input]).await;
            }

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
        file::conversation_file_path(name).unwrap()
    } else {
        println!("What should I save it as?");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        file::conversation_file_path(input.trim()).unwrap()
    };

    client
        .restore_conversation_json(file::main_conversation_file())
        .await?
        .save_history_json(file_name)
        .await?;

    Ok(())
}

async fn remove_conversation(args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Removing conversation {}", name);
        file::conversation_file_path(name).unwrap()
    } else {
        println!("What should I remove?");
        print_saved_conversations();
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        file::conversation_file_path(input.trim()).unwrap()
    };

    std::fs::remove_file(file_name)?;

    Ok(())
}

async fn load_conversation(client: &ChatGPT, args: &[String]) -> chatgpt::Result<()> {
    let file_name = if let Some(name) = args.get(0) {
        println!("Loading conversation from {}", name);
        file::conversation_file_path(name).unwrap()
    } else {
        println!("What should I load it from?");
        print_saved_conversations();
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        file::conversation_file_path(input.trim()).unwrap()
    };

    let conversation = client.restore_conversation_json(file_name).await?;
    conversation
        .save_history_json(file::main_conversation_file())
        .await?;

    Ok(())
}

fn flush_conversation() -> chatgpt::Result<()> {
    std::fs::remove_file(file::main_conversation_file())?;
    Ok(())
}

fn percent_left(current_chunk: &str, chunk_size: usize) -> usize {
    let remaining_size = chunk_size - current_chunk.len();
    (remaining_size as f64 / chunk_size as f64 * 100.0) as usize
}

async fn message_with_file(key: String, args: &[String]) -> chatgpt::Result<()> {
    let file_name = args
        .iter()
        .find(|arg| arg.starts_with("--file="))
        .map(|arg| arg.split_whitespace().nth(0))
        .flatten()
        .map(|arg| arg.trim_start_matches("--file=").to_owned())
        .unwrap();

    let message = args
        .iter()
        .filter(|arg| !arg.starts_with("--file="))
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");

    let expanded_file_name = shellexpand::tilde(&file_name).to_string();
    let absolute_file_path = canonicalize(&expanded_file_name)?;
    let file = File::open(absolute_file_path)?;

    // Read the file line by line
    let reader = BufReader::new(file);
    let mut chunks: Vec<String> = Vec::new();
    let mut current_chunk = String::new();
    let mut current_size = 0;

    let mut is_previous_line_empty = false;
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("Failed to read line: {}", error);
                continue;
            }
        };

        let line_length = line.len();
        if current_size + line_length > CHUNK_SIZE {
            // Push the current chunk into the array and start a new one
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_size = 0;
        }

        current_chunk.push_str(&line);
        current_size += line_length;

        let two_blank_lines = line.trim().is_empty() && is_previous_line_empty;

        if percent_left(&current_chunk, CHUNK_SIZE) < 20 && !two_blank_lines {
            // Push the current chunk into the array and start a new one
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_size = 0;
        }

        is_previous_line_empty = line.trim().is_empty();
    }

    // Push the last chunk into the array if it's not empty
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    // split chunks into a nested array of 5 chunk batches
    let mut batched_chunks: Vec<Vec<String>> = chunks
        .chunks(CHUNK_BATCH_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect();

    let mut results: Vec<String> = Vec::new();

    // Send each chunk in batched_chunks to the AI in sequence
    while let Some(chunk) = batched_chunks.first() {
        let key = key.clone();
        let result = ai::process_chunks(key, message.clone(), chunk.to_vec()).await?;

        for (_index, result) in result.iter().enumerate() {
            results.push(result.message().content.clone());
        }

        batched_chunks.remove(0);
    }

    println!("{}", results.join("\n"));

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

    let conversations = std::fs::read_dir(file::conversations_dir().unwrap())?;
    for conversation in conversations {
        let conversation = conversation?;
        if is_saved_conversation(&conversation) {
            let print_name = conversation.file_name().into_string().unwrap();
            println!(
                "Removing - {}",
                print_name
                    .trim_start_matches("conversation_")
                    .trim_end_matches(".json")
            );
            std::fs::remove_file(conversation.path())?;
        }
    }

    Ok(())
}

async fn process_message(client: &ChatGPT, message: &str) -> chatgpt::Result<()> {
    let mut conversation = if std::path::Path::new(&file::main_conversation_file()).exists() {
        client
            .restore_conversation_json(file::main_conversation_file())
            .await?
    } else {
        client.new_conversation()
    };

    let response = conversation.send_message(message.to_string()).await?;

    // Print two new lines to separate the conversation
    println!("\n\n{}", response.message().content);

    conversation
        .save_history_json(file::main_conversation_file())
        .await?;

    Ok(())
}

fn get_saved_conversations() -> Vec<String> {
    let conversations = std::fs::read_dir(file::conversations_dir().unwrap()).unwrap();
    let mut names: Vec<String> = Vec::new();

    for conversation in conversations {
        if let Ok(conversation) = conversation {
            if is_saved_conversation(&conversation) {
                let print_name = conversation.file_name().into_string().unwrap();
                names.push(
                    print_name
                        .trim_start_matches("conversation_")
                        .trim_end_matches(".json")
                        .to_string(),
                );
            }
        }
    }
    names
}

fn print_saved_conversations() {
    let conversations = std::fs::read_dir(file::conversations_dir().unwrap()).unwrap();
    for conversation in conversations {
        if let Ok(conversation) = conversation {
            if is_saved_conversation(&conversation) {
                let print_name = conversation.file_name().into_string().unwrap();
                println!(
                    "{}",
                    print_name.replace("conversation_", "").replace(".json", "")
                );
            }
        }
    }
}

fn is_saved_conversation(conversation: &std::fs::DirEntry) -> bool {
    let file_name = conversation.file_name().into_string().unwrap();
    file_name.starts_with("conversation_") && file_name.ends_with(".json")
}
