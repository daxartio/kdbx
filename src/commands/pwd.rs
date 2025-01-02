use std::{io, path::PathBuf, thread, time};

use clap::ValueHint;
use keepass::db::Entry;
use log::*;

use crate::{
    clipboard::set_clipboard,
    keepass::{find_entry, get_entries},
    pwd::Pwd,
    utils::{is_tty, open_database_interactively, skim},
    Result, CANCEL, CANCEL_RQ_FREQ,
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Timeout in seconds before clearing the clipboard. 0 means no clean-up
    #[arg(short, long, default_value_t = crate::DEFAULT_TIMEOUT)]
    timeout: u8,

    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Do not ask any interactive question
    #[arg(short = 'n', long)]
    no_interaction: bool,

    /// Preview entry during picking
    #[arg(short = 'v', long)]
    preview: bool,

    /// Use all available screen for picker
    #[arg(short, long)]
    full_screen: bool,

    /// Store password for the database in the OS's keyring
    #[arg(short = 'p', long)]
    use_keyring: bool,

    /// Remove database's password from OS's keyring and exit
    #[arg(short = 'P', long)]
    remove_key: bool,

    /// KDBX file path
    #[arg(short, long, env = "KDBX_DATABASE", value_hint = ValueHint::FilePath)]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE", value_hint = ValueHint::FilePath)]
    key_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.database.exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, _) = open_database_interactively(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
        args.no_interaction,
    )?;

    let query = args.entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let Some(entry) = find_entry(query, &db.root) {
            // Print password to stdout when pipe used
            // e.g. `kdbx pwd example.com | cat`
            if !is_tty(io::stdout()) {
                put!("{}", entry.get_password().unwrap_or_default());
                return Ok(());
            }
            return clip(entry, args.timeout);
        }
    }

    if args.no_interaction {
        return Err("Not found".to_string().into());
    }

    // If more than a single match has been found and stdout is not a TTY
    // than it is not possible to pick the right entry without user's interaction
    if !is_tty(io::stdout()) {
        return Err(format!("No single match for {}.", query.unwrap_or("[empty]")).into());
    }

    if let Some(wrapped_entry) = skim(
        &get_entries(&db.root, ""),
        query.map(String::from),
        args.no_group,
        args.preview,
        args.full_screen,
        false,
    ) {
        clip(wrapped_entry.entry, args.timeout)?
    }

    Ok(())
}

fn clip(entry: &Entry, timeout: u8) -> Result<()> {
    let pwd: Pwd = entry.get_password().unwrap_or_default().to_string().into();

    if set_clipboard(Some(pwd)).is_err() {
        return Err(format!(
            "Clipboard unavailable. Try use STDOUT, i.e. `kdbx pwd '{}' | cat`.",
            entry.get_title().unwrap_or_default()
        )
        .into());
    }

    if timeout == 0 {
        debug!("user decided to leave the password in the buffer");
        return Ok(());
    }

    let mut ticks = u64::from(timeout) * CANCEL_RQ_FREQ;
    while !CANCEL.load(std::sync::atomic::Ordering::SeqCst) && ticks > 0 {
        if ticks % CANCEL_RQ_FREQ == 0 {
            // Note extra space after the "seconds...":
            // transition from XX digits to X digit
            // would shift whole line to the left
            // so extra space's role is to hide a single dot
            put!(
                "Copied to the clipboard! Clear in {} seconds... \x0D",
                ticks / CANCEL_RQ_FREQ
            );
        }
        thread::sleep(time::Duration::from_millis(1_000 / CANCEL_RQ_FREQ));
        ticks -= 1;
    }

    let _ = set_clipboard(None);
    wout!("{:50}", "Wiped out");

    Ok(())
}
