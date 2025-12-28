use anyhow::Result;
use ron::de::from_str;
use std::path::Path;
use teloxide::prelude::*;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::mpsc,
};

use crate::config::Config;

pub const SOCKET_PATH: &str = "/tmp/telecon.sock";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum SocketCommand {
    SendMessage {
        text: Option<String>,
        files: Vec<String>,
        media: Vec<String>,
        buttons: Vec<(String, String)>,
    },
    ReloadServices,
}

pub async fn run(_bot: Bot, _config: Config, tx: mpsc::Sender<SocketCommand>) -> Result<()> {
    run_socket_server(tx).await
}

async fn run_socket_server(tx: mpsc::Sender<SocketCommand>) -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        tokio::fs::remove_file(SOCKET_PATH).await?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;

    loop {
        let (stream, _) = listener.accept().await?;
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, tx).await {
                eprintln!("socket error: {:?}", e);
            }
        });
    }
}

async fn handle_client(stream: UnixStream, tx: mpsc::Sender<SocketCommand>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader.read_line(&mut line).await?;
    let cmd: SocketCommand = from_str(&line.trim())?;

    tx.send(cmd).await.ok();

    writer.write_all(b"ok\n").await?;
    Ok(())
}
