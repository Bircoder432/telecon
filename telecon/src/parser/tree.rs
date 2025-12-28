use super::Service;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Folder(Folder),
    Service(Service),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub title: String,
    pub children: BTreeMap<String, Node>,
}
