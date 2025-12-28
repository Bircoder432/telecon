pub mod handler;
pub mod tree;
pub use handler::*;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    fs,
};

use serde::{Deserialize, Serialize};

use crate::parser::tree::{Folder, Node};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub title: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl Service {
    pub fn load(path: &str) -> Self {
        let content = fs::read_to_string(path).unwrap();
        let service: Service = toml::from_str(&content).unwrap();
        service
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.title, self.name)
    }
}

pub fn parse_tree(path: &str, title: &str) -> Node {
    let mut children = BTreeMap::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path_buf = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if entry.file_type().unwrap().is_dir() {
            let node = parse_tree(path_buf.to_str().unwrap(), &name);
            children.insert(name, node);
        }

        if path_buf.extension().unwrap_or_default() == "toml" {
            let service = Service::load(path_buf.to_str().unwrap());
            children.insert(service.name.clone(), Node::Service(service));
        }
    }

    Node::Folder(Folder {
        title: title.to_string(),
        children,
    })
}
