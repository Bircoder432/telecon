use anyhow::Result;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::parser::tree::Node;

const SOCKET_PATH: &str = "/tmp/telecon.sock";

#[derive(Debug)]
pub enum SocketCommand {
    SendMessage(String),
    ReloadServices,
}

pub async fn run(_services: Node, _config: Config, tx: mpsc::Sender<SocketCommand>) -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        tokio::fs::remove_file(SOCKET_PATH).await?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, tx).await {
                        eprintln!("socket error: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {:?}", e),
        }
    }
}

async fn handle_client(stream: UnixStream, tx: mpsc::Sender<SocketCommand>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    if reader.read_line(&mut line).await? == 0 {
        writer.write_all(b"empty input\n").await?;
        return Ok(());
    }

    let cmd = line.trim();

    if let Some(text) = cmd.strip_prefix("send ") {
        if tx
            .send(SocketCommand::SendMessage(text.to_string()))
            .await
            .is_ok()
        {
            writer.write_all(b"ok\n").await?;
        } else {
            writer.write_all(b"failed to send\n").await?;
        }
    } else if cmd == "reload" {
        if tx.send(SocketCommand::ReloadServices).await.is_ok() {
            writer.write_all(b"reloading services\n").await?;
        } else {
            writer.write_all(b"failed to reload\n").await?;
        }
    } else {
        writer.write_all(b"unknown command\n").await?;
    }

    Ok(())
}
