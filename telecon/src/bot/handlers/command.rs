use super::super::keyboard::folder_keyboard;
use super::super::{Command, Service};
use crate::config;
use crate::parser::tree::Node;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::RwLock;

pub async fn process_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    services: Arc<RwLock<Node>>,
    config: Arc<config::Config>,
) -> ResponseResult<()> {
    if config.owner_id != msg.from().unwrap().id.0 {
        return Ok(());
    }

    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "I'm a service management bot.\nWrite /services",
            )
            .await?;
        }

        Command::Services => {
            let keyboard = folder_keyboard(&*services.read().await, "");

            bot.send_message(msg.chat.id, "Choose a service:")
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
