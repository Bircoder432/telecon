use std::{collections::HashMap, sync::Arc};
use teloxide::payloads::EditMessageTextSetters;

use teloxide::{
    Bot,
    prelude::{Requester, ResponseResult},
    types::CallbackQuery,
};

use crate::{
    bot::keyboard::folder_keyboard,
    config::{self, Config},
    parser::{Service, tree::Node},
    runner,
    utils::find_node,
};

pub async fn process_callback(
    bot: Bot,
    q: CallbackQuery,
    tree: Arc<Node>,
    config: Arc<Config>,
) -> ResponseResult<()> {
    println!(
        "CallbackQuery received: id={} data={:?} from={}",
        q.id, q.data, q.from.id.0
    );

    if q.from.id.0 != config.owner_id {
        return Ok(());
    }

    let data = q.data.unwrap();
    bot.answer_callback_query(q.id).await?;

    if let Some(path) = data.strip_prefix("open:") {
        if let Some(node) = find_node(&tree, path) {
            let kb = folder_keyboard(node, path);
            let msg = q.message.unwrap();

            bot.edit_message_text(msg.chat().id, msg.id(), "Выбери:")
                .reply_markup(kb)
                .await?;
        }
    }

    if let Some(path) = data.strip_prefix("run:") {
        if let Some(Node::Service(service)) = find_node(&tree, path) {
            runner::run_service(service).await;
        }
    }

    Ok(())
}
