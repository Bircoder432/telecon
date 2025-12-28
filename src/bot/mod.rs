mod handlers;
mod keyboard;
use crate::config::{self, Config};
use crate::parser::tree::Node;
use crate::parser::{self, Service};
use crate::socket::SocketCommand;
use handlers::callback;
use handlers::command;
use teloxide::dispatching::UpdateHandler;
use teloxide::dptree::entry;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::{
    Bot,
    dispatching::{DispatcherBuilder, HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Dispatcher,
    types::Update,
    utils::command::BotCommands,
};
use tokio::sync::mpsc;

use std::{collections::HashMap, sync::Arc};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Services,
}

pub async fn run(bot: Bot, services: Node, config: Config, mut rx: mpsc::Receiver<SocketCommand>) {
    let services = Arc::new(tokio::sync::RwLock::new(services));
    let config = Arc::new(config);
    let owner_id: i64 = config
        .clone()
        .owner_id
        .try_into()
        .expect("onwer_id does not fit into i64");
    let handler = entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command::process_command),
        )
        .branch(Update::filter_callback_query().endpoint(callback::process_callback));
    let owner_chat_id: i64 = owner_id.try_into().expect("Error");
    bot.send_message(ChatId(owner_chat_id), "Telecon запущен")
        .await
        .ok();
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![services.clone(), config])
        .build();

    // ⚠️ ВАЖНО
    tokio::select! {
        _ = dispatcher.dispatch() => {}
        _ = socket_rx_loop(bot, rx, owner_id, services.clone(), dirs::data_dir().unwrap().join("telecon").join("services").to_string_lossy().to_string()) => {}
    }
}

async fn socket_rx_loop(
    bot: Bot,
    mut rx: mpsc::Receiver<SocketCommand>,
    owner_id: i64,
    services: Arc<tokio::sync::RwLock<Node>>,
    services_path: String,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            SocketCommand::SendMessage(text) => {
                bot.send_message(ChatId(owner_id), text).await.ok();
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
                        bot.send_message(ChatId(owner_id), "Сервисы обновлены!")
                            .await
                            .ok();
                    }
                    Err(e) => {
                        bot.send_message(
                            ChatId(owner_id),
                            format!("Ошибка при перезагрузке сервисов: {e}"),
                        )
                        .await
                        .ok();
                    }
                }
            }
        }
    }
}
