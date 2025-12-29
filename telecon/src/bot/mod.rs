mod handlers;
mod keyboard;

use crate::config::{self, Config};
use crate::parser::tree::Node;
use crate::parser::{self, HandlerRegistry, Service};
use crate::socket::SocketCommand;
use handlers::{callback, command};
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::{
    dispatching::UpdateHandler,
    dptree::entry,
    prelude::*,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, InputFile},
};
use tokio::sync::{RwLock, mpsc};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Services,
}

pub async fn run(
    bot: Bot,
    services: Arc<RwLock<Node>>,
    config: Config,
    mut rx: mpsc::Receiver<SocketCommand>,
    handlers: Arc<RwLock<HandlerRegistry>>,
) {
    let config = Arc::new(config);
    let owner_id: i64 = config
        .owner_id
        .try_into()
        .expect("owner_id does not fit i64");

    let handler = entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command::process_command),
        )
        .branch(Update::filter_callback_query().endpoint(callback::process_callback));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![
            services.clone(),
            config.clone(),
            handlers.clone()
        ])
        .build();

    bot.send_message(ChatId(owner_id), "Telecon started")
        .await
        .ok();

    tokio::select! {
        _ = dispatcher.dispatch() => {},
        _ = socket_rx_loop(bot.clone(), rx, owner_id, services.clone(), dirs::data_dir().unwrap().join("telecon").join("services").to_string_lossy().to_string(),handlers.clone()) => {},
    }
}

async fn socket_rx_loop(
    bot: Bot,
    mut rx: mpsc::Receiver<SocketCommand>,
    owner_id: i64,
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
                let chat_id = ChatId(owner_id);

                let keyboard = if !buttons.is_empty() {
                    let mut rows = vec![];
                    for (title, callback_data) in buttons {
                        rows.push(vec![InlineKeyboardButton::callback(title, callback_data)]);
                    }
                    Some(InlineKeyboardMarkup::new(rows))
                } else {
                    None
                };

                for file in files {
                    let _ = bot.send_document(chat_id, InputFile::file(file)).await;
                }

                let mut photos = vec![];
                let mut videos = vec![];
                let mut audios = vec![];
                let mut others = vec![];

                for file in media {
                    let ext = std::path::Path::new(&file)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    match ext.as_str() {
                        "jpg" | "jpeg" | "png" | "webp" => photos.push(file),
                        "mp4" | "mov" | "mkv" => videos.push(file),
                        "mp3" | "wav" => audios.push(file),
                        _ => others.push(file),
                    }
                }

                if !photos.is_empty() {
                    let media_group: Vec<teloxide::types::InputMedia> = photos
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let mut media =
                                teloxide::types::InputMediaPhoto::new(InputFile::file(f));
                            if i == photos.len() - 1 {
                                if let Some(t) = &text {
                                    media.caption = Some(t.clone());
                                }
                            }
                            teloxide::types::InputMedia::Photo(media)
                        })
                        .collect();
                    bot.send_media_group(chat_id, media_group).await.ok();
                }

                if !videos.is_empty() {
                    let media_group: Vec<teloxide::types::InputMedia> = videos
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let mut media =
                                teloxide::types::InputMediaVideo::new(InputFile::file(f));
                            if i == videos.len() - 1 && text.is_some() && photos.is_empty() {
                                media.caption = text.clone();
                            }
                            teloxide::types::InputMedia::Video(media)
                        })
                        .collect();
                    bot.send_media_group(chat_id, media_group).await.ok();
                }

                if !audios.is_empty() {
                    for file in audios.clone() {
                        bot.send_audio(chat_id, InputFile::file(file)).await.ok();
                    }

                    if text.is_some() && photos.is_empty() && videos.is_empty() {
                        bot.send_message(chat_id, text.clone().unwrap()).await.ok();
                    }
                }

                for file in others {
                    bot.send_document(chat_id, InputFile::file(file)).await.ok();
                }

                if text.is_some() && photos.is_empty() && videos.is_empty() && audios.is_empty() {
                    let msg_builder = bot.send_message(chat_id, text.unwrap());
                    let msg_builder = if let Some(kb) = keyboard.clone() {
                        msg_builder.reply_markup(kb)
                    } else {
                        msg_builder
                    };
                    msg_builder.await.ok();
                }
            }

            SocketCommand::ReloadServices => {
                match tokio::task::spawn_blocking({
                    let path = services_path.clone();
                    move || parser::parse_tree(&path, "")
                })
                .await
                {
                    Ok(new_tree) => {
                        let mut svc = services.write().await;
                        *svc = new_tree;
                        bot.send_message(ChatId(owner_id), "Services reloaded")
                            .await
                            .ok();
                    }
                    Err(e) => {
                        bot.send_message(
                            ChatId(owner_id),
                            format!("Error reloading services: {e}"),
                        )
                        .await
                        .ok();
                    }
                }
                let handlers_path = dirs::data_dir()
                    .unwrap()
                    .join("telecon")
                    .join("handlers")
                    .to_string_lossy()
                    .to_string();

                let new_registry = parser::load_handlers(&handlers_path);
                let mut handlers_lock = custom_handlers.write().await;
                *handlers_lock = new_registry.clone();
                println!("{:#?}", new_registry);
                bot.send_message(ChatId(owner_id), "Custom handlers reloaded")
                    .await
                    .ok();
            }
        }
    }
}
