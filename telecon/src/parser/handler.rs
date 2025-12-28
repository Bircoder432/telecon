use std::{collections::HashMap, fs};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HandlerConfig {
    pub name: String,
    pub exec: String,
    pub args: Vec<String>,
}
pub struct HandlerRegistry {
    pub handlers: HashMap<String, HandlerConfig>,
}

pub fn load_handlers(path: &str) -> HandlerRegistry {
    let mut registry = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(handler) = toml::from_str::<HandlerConfig>(&content) {
                    registry.insert(handler.name.clone(), handler);
                }
            }
        }
    }

    HandlerRegistry { handlers: registry }
}
