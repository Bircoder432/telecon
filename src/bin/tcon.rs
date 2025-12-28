use clap::{Parser, Subcommand};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Parser)]
#[command(name = "tcon", about = "telecon CLI controller")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a message to Telegram
    Send { text: String },
    /// Reload services from disk
    Reload,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let payload = match cli.command {
        Commands::Send { text } => format!("send {}\n", text),
        Commands::Reload => "reload\n".to_string(),
    };

    let stream = UnixStream::connect("/tmp/telecon.sock").await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    writer.write_all(payload.as_bytes()).await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;
    print!("{line}");

    Ok(())
}
