use clap::{Parser, Subcommand};
use ron::{ser::to_string_pretty, to_string};
use telecon::socket::SocketCommand;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

#[derive(Parser)]
#[command(name = "tcon", about = "telecon CLI controller")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Send {
        #[arg(short = 't', long)]
        text: Option<String>,

        #[arg(short = 'f', long, num_args(0..))]
        files: Vec<String>,

        #[arg(short = 'm', long, num_args(0..))]
        media: Vec<String>,

        #[arg(short, long)]
        buttons: Option<String>,
    },
    Reload,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let cmd = match cli.command {
        Commands::Send {
            text,
            files,
            media,
            buttons,
        } => {
            if text.is_none() && files.is_empty() && media.is_empty() && buttons.is_none() {
                eprintln!("Error: need to specify at least one flag (-t, -f, -m, -b)");
                std::process::exit(1);
            }

            let buttons_parsed = buttons
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| {
                    let mut parts = s.splitn(2, ':');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                })
                .collect::<Vec<_>>();
            println!("{:#?}", buttons_parsed);
            SocketCommand::SendMessage {
                text,
                files: files.into_iter().collect(),
                buttons: buttons_parsed,
                media: media.into_iter().collect(),
            }
        }
        Commands::Reload => SocketCommand::ReloadServices,
    };

    let stream = UnixStream::connect("/tmp/telecon.sock").await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let payload = to_string(&cmd)?;
    writer.write_all(payload.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;
    print!("{line}");

    Ok(())
}
