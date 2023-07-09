use chatgpt::prelude::*;
use super::file;
use std::io::{stdout, Write};
use futures_util::StreamExt;

pub async fn get_client(key: String) -> ChatGPT {
    let config = ModelConfigurationBuilder::default()
        .engine(ChatGPTEngine::Gpt4)
        .build()
        .unwrap();

    let client = match ChatGPT::new_with_config(&key, config) {
        Ok(val) => val,
        Err(err) => panic!("Failed to create ChatGPT client: {}", err),
    };
    return client;
}

pub async fn process_message(client: &ChatGPT, message: &str) -> chatgpt::Result<()> {
    let mut conversation = if std::path::Path::new(&file::main_conversation_file()).exists() {
        client
            .restore_conversation_json(file::main_conversation_file())
            .await?
    } else {
        client.new_conversation()
    };

    let mut stream = conversation
        .send_message_streaming(message.to_string())
        .await?;

    let mut output: Vec<ResponseChunk> = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content {
                delta,
                response_index,
            } => {
                // Printing part of response without the newline
                print!("{delta}");
                // Manually flushing the standard output, as `print` macro does not do that
                stdout().lock().flush().unwrap();
                output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                });
            }
            // We don't really care about other types, other than parsing them into a ChatMessage later
            other => output.push(other),
        }
    }

    // Parsing ChatMessage from the response chunks and saving it to the conversation history
    let messages = ChatMessage::from_response_chunks(output);
    conversation.history.extend(messages);
    // conversation.history.push(messages[0].to_owned());

    conversation
        .save_history_json(file::main_conversation_file())
        .await?;

    Ok(())
}

