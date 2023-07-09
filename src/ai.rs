use chatgpt::Result as ChatGptResult;
use chatgpt::types::CompletionResponse;
use futures::future::{try_join_all, TryFutureExt};
use tokio::task::spawn;

use super::file;
use super::client;


pub async fn process_chunks(key: String, prompt: String, chunks: Vec<String>) -> ChatGptResult<Vec<CompletionResponse>> {
    println!("Processing {} chunks", chunks.len());
    let tasks = chunks.into_iter().map(|chunk| {
        let prompt = prompt.clone();
        let key = key.clone();
        spawn(async move { handle_chunk(key, chunk, prompt).await })
    });


    let responses: Vec<ChatGptResult<CompletionResponse>> = try_join_all(tasks)
        .map_ok(|results| {
            results.into_iter().collect::<Vec<_>>()
        })
        .await.unwrap();

    // change above to a reduce, where errors are dropped
    let mut unwrapped_responses: Vec<CompletionResponse> = Vec::new();
    for res in responses {
        match res {
            Ok(response) => {
                unwrapped_responses.push(response);
            },
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }

    Ok(unwrapped_responses)
}

async fn handle_chunk(key: String, chunk: String, prompt: String) -> ChatGptResult<CompletionResponse> {
    let client = client::get_client(key).await;
    let mut conversation = if std::path::Path::new(&file::main_conversation_file()).exists() {
        client.restore_conversation_json(file::main_conversation_file()).await?
    } else {
        client.new_conversation()
    };
    let message = format!("{}\n\n{}", prompt, chunk);
    let response: CompletionResponse = conversation.send_message(&message).await?;
    Ok(response)
}

