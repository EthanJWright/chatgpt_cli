use std::path::PathBuf;

const APP_NAME: &str = "chatgpt_cli";

pub fn get_data_dir(app_name: &str) -> Option<PathBuf> {
    let data_dir = dirs::home_dir()?.join(".config").join(app_name);
    std::fs::create_dir_all(&data_dir).ok()?;
    Some(data_dir)
}

pub fn conversations_dir() -> Option<PathBuf> {
    return get_data_dir(APP_NAME);
}

pub fn main_conversation_file() -> String {
    return conversations_dir().unwrap().to_string_lossy().to_string() + "/conversation.json";
}

pub fn conversation_file_path(name: &str) -> Option<PathBuf> {
    let conversions_dir = conversations_dir()?;
    Some(PathBuf::from(conversions_dir).join(format!("conversation_{}.json", name)))
}
