use std::sync::Arc;
use tokio::sync::RwLock;

use crate::parser::{self, HandlerRegistry, tree::Node};

pub struct ServiceManager {
    services: Arc<RwLock<Node>>,
    handlers: Arc<RwLock<HandlerRegistry>>,
    services_path: String,
    handlers_path: String,
}

impl ServiceManager {
    pub fn new(
        services: Arc<RwLock<Node>>,
        handlers: Arc<RwLock<HandlerRegistry>>,
        services_path: String,
        handlers_path: String,
    ) -> Self {
        Self {
            services,
            handlers,
            services_path,
            handlers_path,
        }
    }

    pub async fn reload(&self) -> Result<(), String> {
        let new_tree = tokio::task::spawn_blocking({
            let path = self.services_path.clone();
            move || parser::parse_tree(&path, "")
        })
        .await
        .map_err(|e| e.to_string())?;

        *self.services.write().await = new_tree;

        let new_registry = parser::load_handlers(&self.handlers_path);
        *self.handlers.write().await = new_registry;

        Ok(())
    }
}
