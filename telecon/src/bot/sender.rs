use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InputFile, InputMedia, InputMediaPhoto,
        InputMediaVideo,
    },
};
use tokio::sync::RwLock;
use tokio::sync::mpsc::Receiver;

use crate::{
    domain::notification::Notification,
    parser::{HandlerRegistry, tree::Node},
    socket::SocketCommand,
};

pub struct TelegramSender {
    bot: Bot,
    chat_id: ChatId,
}

impl TelegramSender {
    pub fn new(bot: Bot, chat_id: ChatId) -> Self {
        Self { bot, chat_id }
    }

    pub async fn send(&self, n: Notification) {
        for file in &n.files {
            let _ = self
                .bot
                .send_document(self.chat_id, InputFile::file(file))
                .await;
        }

        let mut photos = vec![];
        let mut videos = vec![];
        let mut audios = vec![];
        let mut others = vec![];

        for file in &n.media {
            let ext = std::path::Path::new(file)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                "jpg" | "jpeg" | "png" | "webp" => photos.push(file.clone()),
                "mp4" | "mov" | "mkv" => videos.push(file.clone()),
                "mp3" | "wav" => audios.push(file.clone()),
                _ => others.push(file.clone()),
            }
        }

        let text = n.text.clone();

        if !photos.is_empty() {
            let media_group: Vec<InputMedia> = photos
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let mut m = InputMediaPhoto::new(InputFile::file(f));
                    if i == photos.len() - 1 {
                        if let Some(t) = &text {
                            m.caption = Some(t.clone());
                        }
                    }
                    InputMedia::Photo(m)
                })
                .collect();
            let _ = self.bot.send_media_group(self.chat_id, media_group).await;
        }

        if !videos.is_empty() {
            let media_group: Vec<InputMedia> = videos
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let mut m = InputMediaVideo::new(InputFile::file(f));
                    if i == videos.len() - 1 && text.is_some() && photos.is_empty() {
                        m.caption = text.clone();
                    }
                    InputMedia::Video(m)
                })
                .collect();
            let _ = self.bot.send_media_group(self.chat_id, media_group).await;
        }

        for file in audios.clone() {
            match self
                .bot
                .send_audio(self.chat_id, InputFile::file(&file))
                .await
            {
                Ok(_) => println!("Audio sent: {}", file),
                Err(e) => eprintln!("Failed to send audio {}: {:?}", file, e),
            }
        }

        for file in others {
            let _ = self
                .bot
                .send_document(self.chat_id, InputFile::file(file))
                .await;
        }

        if text.is_some() && photos.is_empty() && videos.is_empty() && audios.is_empty() {
            let mut msg = self.bot.send_message(self.chat_id, text.unwrap());
            if let Some(kb) = self.build_keyboard(&n) {
                msg = msg.reply_markup(kb);
            }
            let _ = msg.await;
        }
    }

    fn build_keyboard(&self, n: &Notification) -> Option<InlineKeyboardMarkup> {
        if n.buttons.is_empty() {
            return None;
        }

        let rows: Vec<Vec<InlineKeyboardButton>> = n
            .buttons
            .iter()
            .map(|(t, d)| vec![InlineKeyboardButton::callback(t, d)])
            .collect();

        Some(InlineKeyboardMarkup::new(rows))
    }
}

pub async fn run_socket_loop(
    bot: TelegramSender,
    mut rx: Receiver<SocketCommand>,
    services: Arc<RwLock<Node>>,
    services_path: String,
    custom_handlers: Arc<RwLock<HandlerRegistry>>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            SocketCommand::SendMessage {
                text,
                files,
                media,
                buttons,
            } => {
                let notification = Notification {
                    text,
                    files,
                    media,
                    buttons,
                };
                bot.send(notification).await;
            }
            SocketCommand::ReloadServices => {
                let new_tree = crate::parser::parse_tree(&services_path, "");
                let mut svc = services.write().await;
                *svc = new_tree;
                bot.send(crate::domain::notification::Notification {
                    text: Some("Services reloaded".into()),
                    files: vec![],
                    media: vec![],
                    buttons: vec![],
                })
                .await;

                let handlers_path = dirs::data_dir()
                    .unwrap()
                    .join("telecon")
                    .join("handlers")
                    .to_string_lossy()
                    .to_string();
                let new_registry = crate::parser::load_handlers(&handlers_path);
                let mut handlers_lock = custom_handlers.write().await;
                *handlers_lock = new_registry.clone();
                bot.send(crate::domain::notification::Notification {
                    text: Some("Custom handlers reloaded".into()),
                    files: vec![],
                    media: vec![],
                    buttons: vec![],
                })
                .await;
            }
        }
    }
}
