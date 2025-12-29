use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::EditMessageTextSetters,
    prelude::{Requester, ResponseResult},
    types::CallbackQuery,
};

use tokio::sync::RwLock;

use crate::{
    app::command_runner::CommandRunner,
    bot::keyboard::folder_keyboard,
    config::Config,
    parser::handler::HandlerConfig,
    parser::{HandlerRegistry, tree::Node},
    utils::find_node,
};

pub async fn process_callback(
    bot: Bot,
    q: CallbackQuery,
    tree: Arc<RwLock<Node>>,
    config: Arc<Config>,
    custom_handlers: Arc<RwLock<HandlerRegistry>>,
) -> ResponseResult<()> {
    let Some(data) = q.data.clone() else {
        return Ok(());
    };

    // кастомные хендлеры
    {
        let handlers = custom_handlers.read().await;
        if let Some(handler) = handlers.handlers.get(&data).cloned() {
            tokio::spawn(async move {
                if let Err(e) = CommandRunner::run(&handler.command, &handler.args).await {
                    eprintln!("Handler `{}` failed: {}", handler.name, e);
                }
            });

            bot.answer_callback_query(q.id).await.ok();
            return Ok(());
        }
    }

    // owner-only
    if q.from.id.0 != config.owner_id {
        return Ok(());
    }

    bot.answer_callback_query(q.id.clone()).await.ok();

    // навигация по дереву
    if let Some(path) = data.strip_prefix("open:") {
        return handle_open(&bot, &q, tree, path).await;
    }

    // запуск сервиса
    if let Some(path) = data.strip_prefix("run:") {
        return handle_run(tree, path).await;
    }

    Ok(())
}

async fn handle_open(
    bot: &Bot,
    q: &CallbackQuery,
    tree: Arc<RwLock<Node>>,
    path: &str,
) -> ResponseResult<()> {
    let tree = tree.read().await;
    let Some(node) = find_node(&tree, path) else {
        return Ok(());
    };

    let kb = folder_keyboard(node, path);
    let Some(msg) = &q.message else {
        return Ok(());
    };

    bot.edit_message_text(msg.chat().id, msg.id(), "Select:")
        .reply_markup(kb)
        .await
        .ok();

    Ok(())
}

async fn handle_run(tree: Arc<RwLock<Node>>, path: &str) -> ResponseResult<()> {
    let tree = tree.read().await;
    let Some(Node::Service(service)) = find_node(&tree, path) else {
        return Ok(());
    };

    if let Err(e) = CommandRunner::run(&service.command, &service.args).await {
        eprintln!("Service `{}` failed: {}", service.name, e);
    }

    Ok(())
}
