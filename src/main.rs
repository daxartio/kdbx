#[macro_use]
mod utils;
mod clipboard;
mod commands;
mod keepass;
mod keyring;
mod logger;
mod pwd;
mod stdin;

use std::{env, error, process, result, sync::atomic, thread, time};

use clap::{Parser, Subcommand};
use log::*;
use once_cell::sync::Lazy;

const DEFAULT_TIMEOUT: u8 = 15;
const CANCEL_RQ_FREQ: u64 = 10;

static BIN_NAME: &str = env!("CARGO_PKG_NAME");
static BIN_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

static CANCEL: atomic::AtomicBool = atomic::AtomicBool::new(false);
static STDIN: Lazy<stdin::Stdin> = Lazy::new(stdin::Stdin::new);

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() {
    logger::init(BIN_NAME);

    set_ctrlc_handler();

    let cli = Cli::parse();

    if let Err(err) = match cli.command {
        Commands::Pwd(args) => commands::pwd::run(args),
        Commands::Totp(args) => commands::totp::run(args),
        Commands::Show(args) => commands::show::run(args),
        Commands::Init(args) => commands::init::run(args),
        Commands::Add(args) => commands::add::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Completion(args) => commands::completion::run(args),
    } {
        werr!("{}", err);
        process::exit(1);
    }
}

#[derive(Parser)]
#[command(version, about = BIN_DESCRIPTION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Copy password and clear clipboard after specified amount of time
    Pwd(commands::pwd::Args),
    /// Copy totp
    Totp(commands::totp::Args),
    /// Display entry's info
    Show(commands::show::Args),
    /// Add new entry
    Add(commands::add::Args),
    /// Init new database
    Init(commands::init::Args),
    /// List all entries
    List(commands::list::Args),
    /// Outputs the completion file for given shell
    Completion(commands::completion::Args),
}

fn set_ctrlc_handler() {
    if let Err(e) = ctrlc::set_handler(|| {
        CANCEL.store(true, atomic::Ordering::SeqCst);
        STDIN.reset_tty();

        // allow gracefully finish any cancellable loop
        thread::sleep(time::Duration::from_millis(2 * 1_000 / CANCEL_RQ_FREQ));

        let _ = clipboard::set_clipboard(None);
        process::exit(1);
    }) {
        warn!("unable to setup Ctrl+C handler: {e}");
    }
}
