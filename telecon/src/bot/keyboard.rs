use crate::parser::tree::{Folder, Node};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn folder_keyboard(node: &Node, path: &str) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    if let Node::Folder(folder) = node {
        for (key, child) in &folder.children {
            match child {
                Node::Folder(f) => {
                    let cb_data = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}/{}", path, key)
                    };

                    rows.push(vec![InlineKeyboardButton::callback(
                        format!("📁 {}", f.title),
                        format!("open:{cb_data}"),
                    )]);
                }
                Node::Service(s) => {
                    let cb_data = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}/{}", path, key)
                    };

                    rows.push(vec![InlineKeyboardButton::callback(
                        format!("⚙️ {}", s.title),
                        format!("run:{cb_data}"),
                    )]);
                }
            }
        }
    }

    if !path.is_empty() {
        let parent = path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
        rows.push(vec![InlineKeyboardButton::callback(
            "⬅ Back",
            format!("open:{parent}"),
        )]);
    }

    InlineKeyboardMarkup::new(rows)
}
