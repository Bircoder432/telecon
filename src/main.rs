mod bot;
mod config;
mod parser;
mod runner;
mod socket;
mod utils;
use parser::Service;
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::load();

    let services_path = dirs::data_dir()
        .unwrap()
        .join("telecon")
        .join("services")
        .to_string_lossy()
        .to_string();

    let services = parser::parse_tree(&services_path, "");

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let bot = teloxide::Bot::new(&config.token);

    let bot_task = {
        let services = services.clone();
        let config = config.clone();

        tokio::spawn(async move {
            bot::run(bot, services, config, rx).await;
        })
    };

    let socket_task = {
        let services = services;
        let config = config.clone();

        tokio::spawn(async move {
            socket::run(services, config, tx).await.unwrap();
        })
    };

    tokio::select! {
        _ = bot_task => eprintln!("bot crashed"),
        _ = socket_task => eprintln!("socket crashed"),
    }

    Ok(())
}
