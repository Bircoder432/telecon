# telecon

telecon is a small Rust project for remote control and monitoring of a Linux machine via a Telegram bot. It provides:

- `telecond` — a background daemon/process intended to run under a regular (unprivileged) user account. It handles Telegram updates, exposes a services menu, and invokes configured handlers.
- `tcon` — a CLI controller used to send messages and trigger callbacks or commands handled by the daemon.

This README explains how to build, configure and use telecon.

## Key concepts

- Services — command entries shown in the bot menu and returned by the `/services` command in the bot. These are the actions a user can select from the bot UI.
- Handlers — executable scripts/programs stored in the handlers directory; they serve as custom callback handlers invoked when `tcon` sends callbacks or when menu items are selected. Handlers receive invocation context (environment variables and/or stdin) from telecond.

## Features

- Runs as a user-level daemon (systemd --user or background process)
- Presents a services menu in the bot (`/services`)
- Send messages, files and media via Telegram
- Interactive buttons (title:callback)
- Reload service definitions and handlers at runtime
- Small Rust implementation focused on reliability and simplicity

## Repository

- Repository: [Bircoder432/telecon](https://github.com/Bircoder432/telecon)

## Build & Install

1. Clone the repository:
```bash
git clone https://github.com/Bircoder432/telecon.git
cd telecon
```

2. Install the two packages from the repository:
```bash
cargo install --path ./telecond
cargo install --path ./tcon
```

This installs `telecond` and `tcon` into your Cargo bin directory (usually `~/.cargo/bin` or `~/.local/cargo/bin`). Ensure that directory is in your PATH, or move the installed binaries to a directory already in PATH (for example `~/.local/bin`).

## Configuration

telecond requires a configuration file containing two fields: `token` and `owner_id`.

Recommended config file locations:
- `$XDG_CONFIG_HOME/telecon/config.toml`
- `~/.config/telecon/config.toml`

Example config:
```toml
# $XDG_CONFIG_HOME/telecon/config.toml
token = "123456:ABC-..."
owner_id = 123456789
```

telecond reads its configuration from the config file at startup. Make sure the config file exists and contains valid values before starting the daemon.

- `token` — your Telegram Bot token.
- `owner_id` — numeric Telegram user id of the owner (used by the daemon to identify the owner).

## Data layout

telecond uses the user data directory to store services and handlers:

- Handlers: `~/.local/share/telecon/handlers`  
  - Place executable scripts or binaries here. These are invoked by telecond for custom callbacks (sent by `tcon`) or other events. Handlers should be executable and accept input via environment variables or stdin as described in the repository examples.

- Services: `~/.local/share/telecon/services`  
  - Place service definitions here. Services are presented in the bot menu and returned when the user sends `/services`. Selecting a service or triggering it will result in telecond invoking the corresponding action (script/handler).

Keep handlers and services next to each other for easy management.

## CLI (tcon) usage

telecon CLI controller

Usage: tcon <COMMAND>

Commands:
  send    Отправить сообщение в Telegram
  reload  Перечитать сервисы
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

Example: the `send` subcommand help output (as produced by the CLI):

vstor@localhost ~> tcon send --help
Отправить сообщение в Telegram

Usage: tcon send [OPTIONS]

Options:
  -t, --text <TEXT>         Текст сообщения
  -f, --files [<FILES>...]  Файлы для отправки (сжатые как документы)
  -m, --media [<MEDIA>...]  Несжатые медиа (отправка как send_document)
  -b, --buttons <BUTTONS>   Кнопки в формате title:callback через запятую
  -h, --help                Print help

Notes:
- `--text` — message text
- `--files` — list of files to send (will be compressed and sent as documents)
- `--media` — list of media files sent uncompressed
- `--buttons` — comma-separated button definitions in the format `Title:callback`

Examples:
```bash
# Send a simple message
tcon send --text "Hello from telecond"

# Send files (compressed as documents)
tcon send --text "Logs" --files /var/log/syslog /var/log/dmesg

# Send uncompressed media
tcon send --text "Screenshot" --media /tmp/screenshot.png

# Send interactive buttons (title:callback)
tcon send --text "Choose action:" --buttons "Reboot:reboot,Update:update"

# Reload services/handlers (forces telecond to re-scan the services and handlers directories)
tcon reload
```

When `tcon` sends a callback (via a button or other mechanism), telecond will invoke the appropriate handler from `~/.local/share/telecon/handlers`.

## Running telecond (user service)

Recommended: run telecond as a systemd user service.

Example `~/.config/systemd/user/telecond.service`:
```ini
[Unit]
Description=telecond - Telegram control daemon (user)

[Service]
Type=simple
ExecStart=%h/.local/bin/telecond
Restart=on-failure

[Install]
WantedBy=default.target
```

Before starting the service, ensure the config file (`$XDG_CONFIG_HOME/telecon/config.toml` or `~/.config/telecon/config.toml`) exists and contains `token` and `owner_id`.

Enable and start:
```bash
systemctl --user daemon-reload
systemctl --user enable --now telecond.service
```

Alternatively run it in background:
```bash
~/.local/bin/telecond &>/var/tmp/telecond.log &
```
(ensure the config file is present and readable by the user running the process).

## Handlers and services details

- Handlers in `~/.local/share/telecon/handlers` must be executable. telecond will call them when corresponding callbacks are received. Handlers may receive context through environment variables and/or stdin; see repository examples for the exact contract.
- Services in `~/.local/share/telecon/services` define the entries shown by the bot `/services` menu. Each service should map to an action that telecond can perform (for example, running a script or returning status).

Check the repository examples for handler and service file formats.

## Security and privacy

- Keep your bot token secret.
- Run telecond under an unprivileged user and review handler scripts carefully.
- Restrict file permissions on handlers and service definitions.

## Extending and customization

- Add or modify handlers in `~/.local/share/telecon/handlers`.
- Add or modify service definitions in `~/.local/share/telecon/services`.
- Implement additional checks or authorization logic in telecond code if required for your environment.

## Contributing

Contributions are welcome. Open issues or PRs on the repository. For code changes, fork the repo, create a branch, and open a PR.

## License

MIT — see the LICENSE file for details.

## Contact

Open an issue or contact the repository owner: [Bircoder432](https://github.com/Bircoder432).
