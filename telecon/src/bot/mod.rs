mod handlers;
mod keyboard;
pub mod sender;

use crate::parser::HandlerRegistry;
use crate::parser::tree::Node;
use crate::socket::SocketCommand;
use crate::{bot::sender::TelegramSender, config::Config};

use handlers::{callback, command};

use std::sync::Arc;
use teloxide::{dispatching::UpdateHandler, dptree::entry, prelude::*, types::ChatId};
use tokio::sync::{RwLock, mpsc};

#[derive(teloxide::macros::BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Services,
}

pub async fn run(
    bot: Bot,
    services: Arc<RwLock<Node>>,
    config: Config,
    rx: mpsc::Receiver<SocketCommand>,
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
    let sender = TelegramSender::new(bot.clone(), ChatId(owner_id));
    let services_path = dirs::data_dir()
        .unwrap()
        .join("telecon")
        .join("services")
        .to_string_lossy()
        .to_string();
    tokio::select! {
        _ = dispatcher.dispatch() => {},
        _ = sender::run_socket_loop(sender, rx, services.clone(), services_path, handlers.clone()) => {},
    }
}
