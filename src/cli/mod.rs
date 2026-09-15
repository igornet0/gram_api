//! CLI command tree (clap). Thin layer over `control` + `server`.

use std::path::PathBuf;
use std::process::ExitCode as StdExit;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use crate::control::{BotService, ControlError, ExitCode, GramConfig, GramPaths};

#[cfg(feature = "accounts")]
use crate::control::{AccountService, LoginRequest};

#[derive(Debug, Parser)]
#[command(
    name = "gram",
    version,
    about = "Control plane for gram_api (bots + optional accounts)"
)]
pub struct Cli {
    /// Data directory (default: ~/.gram)
    #[arg(long, global = true, env = "GRAM_DATA_DIR")]
    pub data: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Telegram User Accounts (requires `--features accounts`)
    Acc {
        #[command(subcommand)]
        cmd: AccCmd,
    },
    /// Telegram Bots (Bot API)
    Bot {
        #[command(subcommand)]
        cmd: BotCmd,
    },
    /// Start local services
    Start {
        #[command(subcommand)]
        cmd: StartCmd,
    },
    /// Summary status
    Status,
    /// Version info
    Version,
}

#[derive(Debug, Subcommand)]
pub enum AccCmd {
    List,
    Login {
        /// Phone number (+7…)
        phone: Option<String>,
        #[arg(long = "phone")]
        phone_opt: Option<String>,
        /// Auth code (sensitive; prefer interactive prompt)
        #[arg(long)]
        code: Option<String>,
    },
    Logout {
        account: String,
    },
    Start {
        account: String,
    },
    Stop {
        account: String,
    },
    Status {
        account: String,
    },
    Info {
        account: String,
    },
    Remove {
        account: String,
    },
    Send {
        account: String,
        chat: String,
        message: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BotCmd {
    List,
    Add {
        name: Option<String>,
        #[arg(long = "name")]
        name_opt: Option<String>,
    },
    Remove {
        name: String,
    },
    Start {
        name: String,
    },
    Stop {
        name: String,
    },
    Status {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum StartCmd {
    /// Local Web UI (127.0.0.1 by default)
    Ui {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
}

pub async fn run() -> StdExit {
    let cli = Cli::parse();
    match run_inner(cli).await {
        Ok(()) => StdExit::from(ExitCode::Success as u8),
        Err(e) => {
            eprintln!("Error: {e}");
            StdExit::from(ExitCode::from_error(&e) as u8)
        }
    }
}

async fn run_inner(cli: Cli) -> Result<(), ControlError> {
    let root = GramConfig::resolve_data_dir(cli.data.clone());
    let paths = GramPaths::new(root);
    paths.ensure()?;
    let mut config = GramConfig::load(&paths)?;

    match cli.command {
        Commands::Version => {
            println!("gram {}", env!("CARGO_PKG_VERSION"));
            println!("gram_api {}", env!("CARGO_PKG_VERSION"));
            println!(
                "accounts {}",
                if cfg!(feature = "accounts") {
                    "enabled (TDLib linked)"
                } else {
                    "disabled (rebuild with --features accounts)"
                }
            );
            Ok(())
        }
        Commands::Status => cmd_status(&paths, &config).await,
        Commands::Bot { cmd } => {
            let bots = BotService::new(paths.clone(), config.clone())?;
            cmd_bot(cmd, &bots).await
        }
        Commands::Start { cmd } => cmd_start(cmd, paths, &mut config).await,
        Commands::Acc { cmd } => {
            #[cfg(feature = "accounts")]
            {
                let accounts = AccountService::new(paths.clone(), config.clone())?;
                cmd_acc(cmd, &accounts).await
            }
            #[cfg(not(feature = "accounts"))]
            {
                let _ = cmd;
                Err(ControlError::Unavailable(
                    "`gram acc` requires a build with --features accounts (TDLib)".into(),
                ))
            }
        }
    }
}

async fn cmd_status(paths: &GramPaths, config: &GramConfig) -> Result<(), ControlError> {
    let bots = BotService::new(paths.clone(), config.clone())?;
    let bot_list = bots.list()?;
    let running = bot_list.iter().filter(|b| b.desired_running).count();
    println!("Gram");
    println!("Data        {}", paths.root().display());
    println!("Bots        {}", bot_list.len());
    println!("  Running   {running}");
    println!("  Stopped   {}", bot_list.len() - running);
    #[cfg(feature = "accounts")]
    {
        let accounts = AccountService::new(paths.clone(), config.clone())?;
        let list = accounts.list()?;
        println!("Accounts    {}", list.len());
    }
    #[cfg(not(feature = "accounts"))]
    {
        println!("Accounts    (feature disabled)");
    }
    println!("UI          stopped (use: gram start ui)");
    Ok(())
}

async fn cmd_bot(cmd: BotCmd, bots: &BotService) -> Result<(), ControlError> {
    match cmd {
        BotCmd::List => {
            let list = bots.list()?;
            if list.is_empty() {
                println!("No bots. Add one with: gram bot add");
                return Ok(());
            }
            println!("{:<16} {:<16} RUNNING", "NAME", "USERNAME");
            for b in list {
                println!(
                    "{:<16} {:<16} {}",
                    b.name,
                    b.username.as_deref().unwrap_or("-"),
                    if b.desired_running { "yes" } else { "no" }
                );
            }
            Ok(())
        }
        BotCmd::Add { name, name_opt } => {
            let name = name_opt
                .or(name)
                .ok_or_else(|| ControlError::Invalid("usage: gram bot add <name>".into()))?;
            eprint!("Bot token: ");
            let token = rpassword::read_password()
                .map_err(|e| ControlError::Io(e.to_string()))?
                .trim()
                .to_string();
            let rec = bots.add(&name, &token)?;
            println!("✓ Bot added: {}", rec.name);
            Ok(())
        }
        BotCmd::Remove { name } => {
            bots.remove(&name)?;
            println!("✓ Removed bot `{name}`");
            Ok(())
        }
        BotCmd::Start { name } => {
            let st = bots.start(&name).await?;
            println!(
                "✓ Bot `{}` verified (reachable={:?})",
                st.name, st.reachable
            );
            Ok(())
        }
        BotCmd::Stop { name } => {
            bots.stop(&name).await?;
            println!("✓ Bot `{name}` marked stopped");
            Ok(())
        }
        BotCmd::Status { name } => {
            let st = bots.status(&name).await?;
            println!("Bot: {}", st.name);
            println!("Username     {}", st.username.as_deref().unwrap_or("-"));
            println!("Running      {}", st.desired_running);
            println!("Token        {}", st.token_masked);
            println!("Reachable    {:?}", st.reachable);
            Ok(())
        }
    }
}

async fn cmd_start(
    cmd: StartCmd,
    paths: GramPaths,
    config: &mut GramConfig,
) -> Result<(), ControlError> {
    match cmd {
        StartCmd::Ui { host, port } => {
            if let Some(h) = host {
                config.ui.host = h;
            }
            if let Some(p) = port {
                config.ui.port = p;
            }
            // Force localhost unless explicitly overridden already in config/CLI.
            if config.ui.host == "0.0.0.0" {
                eprintln!("Warning: binding 0.0.0.0 exposes UI on all interfaces");
            }
            let bots = BotService::new(paths.clone(), config.clone())?;
            #[cfg(feature = "accounts")]
            let accounts = AccountService::new(paths.clone(), config.clone())?;
            let state = Arc::new(crate::server::AppState {
                paths: paths.clone(),
                config: config.clone(),
                bots,
                #[cfg(feature = "accounts")]
                accounts,
            });
            let url = format!("http://{}:{}", config.ui.host, config.ui.port);
            println!("Gram UI");
            println!("Server: {url}");
            println!("✓ API started");
            println!("✓ Web UI started");
            println!("Open {url}");
            crate::server::serve(state, &config.ui.host, config.ui.port).await
        }
    }
}

#[cfg(feature = "accounts")]
async fn cmd_acc(cmd: AccCmd, accounts: &AccountService) -> Result<(), ControlError> {
    match cmd {
        AccCmd::List => {
            let list = accounts.list()?;
            if list.is_empty() {
                println!("No accounts. Login with: gram acc login +7…");
                return Ok(());
            }
            println!("{:<16} {:<16} NAME", "ID", "PHONE");
            for a in list {
                println!(
                    "{:<16} {:<16} {}",
                    a.key,
                    a.phone,
                    a.display_name.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
        AccCmd::Login {
            phone,
            phone_opt,
            code,
        } => {
            let phone = phone_opt
                .or(phone)
                .ok_or_else(|| ControlError::Invalid("usage: gram acc login <phone>".into()))?;
            println!("Telegram account login");
            println!("Phone: {phone}");
            let req = LoginRequest {
                phone,
                code,
                password: None,
                first_name: None,
            };
            let mut prompt_code = || {
                eprint!("Code: ");
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| ControlError::Io(e.to_string()))?;
                Ok(line.trim().to_string())
            };
            let mut prompt_password = || {
                eprint!("2FA password: ");
                rpassword::read_password().map_err(|e| ControlError::Io(e.to_string()))
            };
            let mut prompt_name = || {
                eprint!("First name: ");
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| ControlError::Io(e.to_string()))?;
                Ok(line.trim().to_string())
            };
            let res = accounts
                .login(
                    req,
                    crate::control::LoginPrompts::Interactive {
                        code: &mut prompt_code,
                        password: &mut prompt_password,
                        first_name: &mut prompt_name,
                    },
                )
                .await?;
            match res.phase {
                crate::control::AuthPhase::Authorized => {
                    println!("✓ Account authorized");
                    if let Some(v) = res.view {
                        println!("✓ Account: {}", v.phone);
                        println!("✓ Data: {}", v.data_dir);
                    }
                }
                other => {
                    println!("Phase: {other:?}");
                    println!("{}", res.message);
                }
            }
            Ok(())
        }
        AccCmd::Logout { account } => {
            accounts.logout(&account).await?;
            println!("✓ Logged out `{account}`");
            Ok(())
        }
        AccCmd::Start { account } => {
            let v = accounts.start(&account).await?;
            println!("✓ Started `{}` ({})", v.key, v.status);
            Ok(())
        }
        AccCmd::Stop { account } => {
            accounts.stop(&account).await?;
            println!("✓ Stopped `{account}`");
            Ok(())
        }
        AccCmd::Status { account } => {
            let v = accounts.status(&account).await?;
            println!("Account: {}", v.key);
            println!("Phone        {}", v.phone);
            println!("Status       {}", v.status);
            println!("Name         {}", v.display_name.as_deref().unwrap_or("-"));
            println!("Data         {}", v.data_dir);
            Ok(())
        }
        AccCmd::Info { account } => {
            let rec = accounts.info(&account)?;
            println!("{rec:#?}");
            Ok(())
        }
        AccCmd::Remove { account } => {
            accounts.remove(&account).await?;
            println!("✓ Removed `{account}`");
            Ok(())
        }
        AccCmd::Send {
            account,
            chat,
            message,
        } => {
            let id = accounts.send(&account, &chat, &message).await?;
            println!("✓ Sent message id={id}");
            Ok(())
        }
    }
}
