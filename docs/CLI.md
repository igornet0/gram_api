# CLI (`gram`)

Control plane for `gram_api`. Thin layer over library Bot / Account APIs.

## Install

```bash
# Bot-only CLI + UI
cargo install --path . --bin gram

# Bot + user accounts (links TDLib / tdjson)
cargo install --path . --bin gram --features accounts
```

## Layout

```
~/.gram/
├── config.toml
├── accounts/<phone>/     # only with --features accounts
├── bots/<name>/
│   ├── bot.toml
│   └── token             # mode 0600
├── logs/
└── ui/
```

Override root: `gram --data ./data …` or `GRAM_DATA_DIR`.

## Commands

```bash
gram version
gram status
gram bot list|add|remove|start|stop|status
gram start ui [--host 127.0.0.1] [--port 8788]
gram acc …   # requires --features accounts
```

### Accounts (feature `accounts`)

```bash
# config.toml needs [telegram] api_id / api_hash (or TELEGRAM_API_* env)
gram acc login +79854444444
gram acc list
gram acc status 79854444444
gram acc send 79854444444 @channel "hello"
```

Auth code / 2FA are prompted interactively (or `GRAM_AUTH_CODE` / `GRAM_2FA_PASSWORD`). Never pass 2FA on argv.

## Architecture

CLI and Web UI share `gram_api::control` (`BotService`, `AccountService`).
