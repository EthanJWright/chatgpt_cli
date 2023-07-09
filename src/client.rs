use chatgpt::prelude::*;

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
