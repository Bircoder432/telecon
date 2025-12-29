use std::sync::Arc;
use teloxide::{
    Bot,
    payloads::EditMessageTextSetters,
    prelude::{Requester, ResponseResult},
    types::CallbackQuery,
};
use tokio::sync::RwLock;

use crate::{
    bot::keyboard::folder_keyboard,
    config::{self, Config},
    parser::{HandlerRegistry, tree::Node},
    runner,
    utils::find_node,
};

pub async fn process_callback(
    bot: Bot,
    q: CallbackQuery,
    tree: Arc<RwLock<Node>>,
    config: Arc<Config>,
    custom_handlers: Arc<RwLock<HandlerRegistry>>,
) -> ResponseResult<()> {
    println!(
        "CallbackQuery received: id={} data={:?} from={}",
        q.id, q.data, q.from.id.0
    );

    let data = match &q.data {
        Some(d) => d.clone(),
        None => return Ok(()),
    };

    {
        let handlers = custom_handlers.read().await;
        if let Some(handler) = handlers.handlers.get(&data) {
            println!("Found handler: {:?}", handler);
            let handler = handler.clone();
            tokio::spawn(async move {
                crate::runner::run_custom_handler(&handler).await;
            });
            bot.answer_callback_query(q.id).await.ok();
            return Ok(());
        } else {
            println!("No handler found for data: {:?}", data);
        }
    }

    if q.from.id.0 != config.owner_id {
        return Ok(());
    }

    bot.answer_callback_query(q.id).await?;

    if let Some(path) = data.strip_prefix("open:") {
        let tree = tree.read().await;
        if let Some(node) = find_node(&tree, path) {
            let kb = folder_keyboard(node, path);
            if let Some(msg) = q.message {
                bot.edit_message_text(msg.chat().id, msg.id(), "Select:")
                    .reply_markup(kb)
                    .await
                    .ok();
            }
        }
    }

    if let Some(path) = data.strip_prefix("run:") {
        let tree = tree.read().await;
        if let Some(node) = find_node(&tree, path) {
            if let Node::Service(service) = node {
                runner::run_service(service).await;
            }
        }
    }

    Ok(())
}
