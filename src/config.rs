use std::{fs, error::Error};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Deserialize, Serialize)]
pub(crate) struct Config {
    port: u16,
    mode: String,
    backup_dir: String,
    max_elements: usize,
    delete_old_when_full: bool,
    keep_time: u64,
    max_folder_size_gb: f32,
    admin_key: String,
    public_key: String,
    accept_keys: Vec<String>,
}

impl Config {
    pub(crate) fn load() {}

    fn parse() -> Result<Self, Box<dyn Error>> {
        let raw = fs::read_to_string("./config.json")?;
        let config: Self = serde_json::from_str(&raw)?;
        match config.mode.as_str() {
            "storage" => {},
            "client" => {},
            "both" => {},
            _ => {},
        }
        Ok(config)
    }
}
