# tglib

Telegram library for Rust: **User / TDLib** accounts and **Bot API**.

TDLib JSON `@type` names never leave `tglib-tdlib`.

```rust
use tglib::{MockTelegramClient, TelegramAccountId, TelegramClient};
use tglib::bot::BotClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MockTelegramClient::new(TelegramAccountId::new(), "/tmp/tglib-demo".into());
    client.start().await?;
    client.submit_phone("+10000000000".into()).await?;
    client.submit_code("12345".into()).await?;

    let bot = BotClient::new("123456:ABC")?.with_api_base("https://api.telegram.org");
    let _me = bot.get_me().await?;
    Ok(())
}
```

## Crates

| Crate | Role |
|-------|------|
| `tglib` | Public facade (`pub use` of the crates below) |
| `tglib-core` | IDs, account status, events, permissions, errors |
| `tglib-client` | `TelegramClient` trait, DTOs, mock, Markdown→HTML |
| `tglib-tdlib` | TDLib adapter (stub by default; `native` links `tdjson`) |
| `tglib-bot` | HTTP Bot API client (`getMe`, `sendMessage`, `sendPhoto`, `sendDocument`) |

## Features

```bash
# Default / CI — stub, no tdjson
cargo test --workspace

# Live TDLib
cargo test -p tglib-tdlib --features native
# or via facade:
cargo test -p tglib --features native
```

## User credentials (TDLib)

Register an app at https://my.telegram.org, then either:

```bash
cp telegram.toml.example telegram.toml
# fill api_id / api_hash
```

or:

```bash
export TELEGRAM_API_ID=…
export TELEGRAM_API_HASH=…
```

Optional: `TGLIB_TELEGRAM_CONFIG=/path/to/telegram.toml`,  
`TGLIB_EMBEDDED_TELEGRAM_API_ID` / `_HASH` (compile-time),  
`TDLIB_DIR` / `TDLIB_LIB_DIR` for a custom TDLib install.

Session files belong to the host app under a directory you pass into the adapter/factory (e.g. `{data}/accounts/<id>/`).

## Bot credentials

Pass a BotFather token into `BotClient::new(token)`. Optional `with_api_base` (default `https://api.telegram.org`). Tokens must never be logged.

## API surface

User:

- Auth: phone / code / password / `register_user` (`WaitRegistration`)
- Chats: list / search / get / join public / leave
- Messages: get / send text / forward / edit / delete
- Media: `send_photo`, `send_document`, `download_file`
- Updates: new / edited / deleted messages, chat title, auth, connection

Bot:

- `get_me`, `send_message` (parse modes via `prepare_telegram_text`), `send_photo`, `send_document`

## Out of scope

- Bot long-polling / webhook dispatcher / FSM
- Multi-account SQLite stores, product REST/WS, UI

## Layout

```
tglib/
├── Cargo.toml
├── telegram.toml.example
└── crates/
    ├── tglib/
    ├── tglib-core/
    ├── tglib-client/
    ├── tglib-tdlib/
    └── tglib-bot/
```
