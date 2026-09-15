//! TDLib user account example (requires `--features accounts` and `tdjson`).
//!
//! ```bash
//! export TELEGRAM_API_ID=…
//! export TELEGRAM_API_HASH=…
//! cargo run --example account --features accounts -- +10000000000
//! ```

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use gram_api::account::{Account, TdlibCredentials, TelegramAccountStatus};
use gram_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let phone = env::args()
        .nth(1)
        .expect("Usage: account --features accounts -- <PHONE>");

    let api_id: i32 = env::var("TELEGRAM_API_ID")?.parse()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;

    let dir = PathBuf::from("./data/account-example");
    let account = Account::new(
        &dir,
        TdlibCredentials {
            api_id,
            api_hash,
            use_test_dc: false,
        },
    );

    account.start().await?;
    println!("status: {}", account.auth_state().await);

    if account.auth_state().await == TelegramAccountStatus::WaitPhoneNumber {
        account.submit_phone(phone).await?;
    }

    loop {
        match account.auth_state().await {
            TelegramAccountStatus::WaitCode => {
                let code = prompt("code: ")?;
                account.submit_code(code).await?;
            }
            TelegramAccountStatus::WaitPassword => {
                let password = prompt("2FA password: ")?;
                account.submit_password(password).await?;
            }
            TelegramAccountStatus::WaitRegistration => {
                let first = prompt("first name: ")?;
                account.register_user(first, None).await?;
            }
            TelegramAccountStatus::Ready => break,
            other => {
                println!("status: {other}");
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }

    let profile = account.profile().await?;
    println!("ready: {profile:?}");

    // Bot API still available in the same crate:
    let _bot = Bot::new().token("0123456789:DEMO");
    Ok(())
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}
