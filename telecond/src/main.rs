use std::io::Write;
use std::sync::Arc;
use telecon::bot;
use telecon::config;
use telecon::parser;
use telecon::parser::HandlerConfig;
use telecon::parser::Service;
use telecon::parser::load_handlers;
use telecon::socket;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::load();

    let services_path = dirs::data_dir()
        .unwrap()
        .join("telecon")
        .join("services")
        .to_string_lossy()
        .to_string();

    let services_tree = parser::parse_tree(&services_path, "");
    let services = Arc::new(RwLock::new(services_tree));
    let handlers_path = dirs::data_dir()
        .unwrap()
        .join("telecon")
        .join("handlers")
        .to_string_lossy()
        .to_string();
    let handlers = Arc::new(RwLock::new(load_handlers(&handlers_path)));
    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let bot = teloxide::Bot::new(&config.token);
    let bot_clone = bot.clone();

    let bot_task = {
        let services = services.clone();
        let config = config.clone();
        let rx = rx;

        tokio::spawn(async move {
            bot::run(bot_clone, services, config, rx, handlers).await;
        })
    };

    let socket_task = {
        let services = services.clone();
        let config = config.clone();
        let tx = tx;

        tokio::spawn(async move {
            socket::run(bot, config, tx).await.unwrap();
        })
    };

    tokio::select! {
        _ = bot_task => eprintln!("bot crashed"),
        _ = socket_task => eprintln!("socket crashed"),
    }

    Ok(())
}
