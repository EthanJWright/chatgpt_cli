use chatgpt::prelude::*;
use chatgpt::types::CompletionResponse;
use std::env::args;
use std::io::{stdin, stdout, Write};

const CONVERSATION: &str = "conversation.json";

#[tokio::main]
async fn main() -> chatgpt::Result<()> {
    let key = args().nth(1).unwrap();
    let client = ChatGPT::new(&key)?;

    print!("Enter your command: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    match input.trim() {
        "flush" => flush_conversation(),
        "save" => save_conversation(&client).await,
        "load" => load_conversation(&client).await,
        "clear" => clear_conversations(),
        message => process_message(&client, message).await,
    }
}

async fn save_conversation(client: &ChatGPT) -> chatgpt::Result<()> {
    print!("What should I save it as?: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let file_name = format!("conversation{}.json", input.trim());

    client
        .restore_conversation_json(CONVERSATION)
        .await?
        .save_history_json(file_name)
        .await?;

    std::fs::remove_file(CONVERSATION)?;

    Ok(())
}

async fn load_conversation(client: &ChatGPT) -> chatgpt::Result<()> {
    print!("What should I load it from?");
    stdout().flush()?;
    print_saved_conversations();
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let file_name = format!("conversation{}.json", input.trim());

    let conversation: Conversation = client.restore_conversation_json(file_name).await?;
    conversation.save_history_json(CONVERSATION).await?;

    Ok(())
}

fn flush_conversation() -> chatgpt::Result<()> {
    std::fs::remove_file(CONVERSATION)?;

    Ok(())
}

fn clear_conversations() -> chatgpt::Result<()> {
    let mut conversations = std::fs::read_dir(".")?;

    while let Some(conversation) = conversations.next() {
        let conversation = conversation?;

        if is_saved_conversation(&conversation) {
            let print_name = conversation.file_name().into_string().unwrap();
            println!("Removing - {}", print_name.replace("conversation", "").replace(".json", ""));
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
    println!("Response: {}", response.message().content);
    conversation.save_history_json(CONVERSATION).await?;

    Ok(())
}

fn print_saved_conversations() {
    let conversations = std::fs::read_dir(".").unwrap();

    for conversation in conversations {
        if let Ok(conversation) = conversation {
            if is_saved_conversation(&conversation) {
                let print_name = conversation.file_name().into_string().unwrap();
                println!("{}", print_name.replace("conversation", "").replace(".json", ""));
            }
        }
    }
}

fn is_saved_conversation(conversation: &std::fs::DirEntry) -> bool {
    let file_name = conversation.file_name().into_string().unwrap();

    conversation.file_type().unwrap().is_file()
        && file_name.starts_with("conversation")
        && file_name.ends_with(".json")
}


