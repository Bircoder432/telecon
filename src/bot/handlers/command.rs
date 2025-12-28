use super::super::keyboard::folder_keyboard;
use super::super::{Command, Service};
use crate::config;
use crate::parser::tree::Node;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn process_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    services: Arc<Node>,
    config: Arc<config::Config>,
) -> ResponseResult<()> {
    if config.owner_id != msg.from().unwrap().id.0 {
        return Ok(());
    }

    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "Я бот для управления сервисами.\nНапиши /services",
            )
            .await?;
        }

        Command::Services => {
            let keyboard = folder_keyboard(&services.clone(), "");

            bot.send_message(msg.chat.id, "Выбирай сервис:")
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
