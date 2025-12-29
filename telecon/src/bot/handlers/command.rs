use crate::config::Config;
use crate::parser::tree::Node;

use super::super::Command;
use super::super::keyboard::folder_keyboard;

use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::RwLock;

pub async fn process_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    services: Arc<RwLock<Node>>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    // 🔐 Проверка владельца без паники
    let Some(from) = msg.from() else {
        return Ok(());
    };

    if from.id.0 != config.owner_id {
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
            let tree = services.read().await;
            let keyboard = folder_keyboard(&tree, "");

            bot.send_message(msg.chat.id, "Choose a service:")
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
