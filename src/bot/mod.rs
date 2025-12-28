mod handlers;
mod keyboard;
use crate::config;
use crate::parser::tree::Node;
use crate::parser::{self, Service};
use handlers::callback;
use handlers::command;
use teloxide::dispatching::UpdateHandler;
use teloxide::dptree::entry;
use teloxide::{
    Bot,
    dispatching::{DispatcherBuilder, HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Dispatcher,
    types::Update,
    utils::command::BotCommands,
};

use std::{collections::HashMap, sync::Arc};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Services,
}

pub async fn run(services: Node, config: config::Config) {
    let bot = Bot::new(&config.token);

    let services = Arc::new(services);
    let config = Arc::new(config);

    let handler: UpdateHandler<_> = entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command::process_command),
        )
        .branch(Update::filter_callback_query().endpoint(callback::process_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![services, config])
        .build()
        .dispatch()
        .await;
}
