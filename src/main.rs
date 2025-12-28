mod bot;
mod config;
mod parser;
mod runner;
mod utils;
use parser::Service;
use std::io::Write;

#[tokio::main]
async fn main() {
    let config = config::Config::load();
    let services_path = dirs::data_dir()
        .unwrap()
        .join("telecon")
        .join("services")
        .to_str()
        .unwrap()
        .to_string();
    let services = parser::parse_tree(&services_path, "");
    bot::run(services, config).await;
}
