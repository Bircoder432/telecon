use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub owner_id: u64,
}

impl Config {
    pub fn load() -> Config {
        let config_path = dirs::config_dir()
            .unwrap()
            .join("telecon")
            .join("config.toml");
        let content = fs::read_to_string(config_path).unwrap();
        let config: Config = toml::from_str(&content).unwrap();
        config
    }
}
