# gram_api 

aiogram-like Telegram library for Rust + **`gram` CLI** control plane.

## Build flavors

| Flavor | Features | TDLib | CLI |
|--------|----------|-------|-----|
| Bot only | `cli` (default) | no | `gram bot`, `gram start ui`, … |
| Bot + accounts | `cli,accounts` | linked | + `gram acc …` |

```toml
gram_api = { path = "…" }                       # library, bots
gram_api = { path = "…", features = ["accounts"] } # + Account / TDLib
```

```bash
cargo install --path . --bin gram
cargo install --path . --bin gram --features accounts
```

## Library

```rust
use gram_api::prelude::*;
let bot = Bot::new().token(token).workers(4);

#[cfg(feature = "accounts")]
{
    use gram_api::{Account, TdlibCredentials};
    let account = Account::new("./data/acc1", TdlibCredentials { /* … */ });
}
```

## CLI

```bash
gram --help
gram version
gram status
gram bot add mybot          # prompts for token (hidden)
gram bot list
gram start ui               # http://127.0.0.1:8788

# with --features accounts
gram acc login +79854444444
gram acc list
```

Data: `~/.gram` (override with `--data` / `GRAM_DATA_DIR`).

See [docs/CLI.md](docs/CLI.md) and [docs/UI.md](docs/UI.md).

## Architecture

```
gram CLI ──┐
           ├── control::{BotService, AccountService}
gram UI  ──┘
                │
         gram_api Bot / Account
                │
           Bot API / TDLib
```
