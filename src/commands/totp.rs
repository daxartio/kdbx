use std::{io, path::PathBuf};

use clap::ValueHint;
use keepass::db::Entry;

use crate::{
    Result,
    clipboard::set_clipboard,
    keepass::{find_entry, get_entries},
    pwd::Pwd,
    utils::{is_tty, open_database_interactively, skim},
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Do not ask any interactive question
    #[arg(short = 'n', long)]
    no_interaction: bool,

    /// Preview entry during picking
    #[arg(short = 'v', long)]
    preview: bool,

    /// Show the secret instead of code
    #[arg(long)]
    raw: bool,

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

    if let Some(query) = query
        && let Some(entry) = find_entry(query, &db.root)
    {
        // Print totp to stdout when pipe used
        // e.g. `kdbx totp example.com | cat`
        if !is_tty(io::stdout()) {
            let totp = get_totp(entry, args.raw)?;
            put!("{}", totp.as_ref());
            return Ok(());
        }
        return clip(entry, args.raw);
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
        true,
    ) {
        clip(wrapped_entry.entry, args.raw)?
    }

    Ok(())
}

fn clip(entry: &Entry, raw: bool) -> Result<()> {
    let totp = get_totp(entry, raw)?;
    if set_clipboard(Some(totp)).is_err() {
        return Err(format!(
            "Clipboard unavailable. Try use STDOUT, i.e. `kdbx totp '{}' | cat`.",
            entry.get_title().unwrap_or_default()
        )
        .into());
    }

    Ok(())
}

fn get_totp(entry: &Entry, raw: bool) -> Result<Pwd> {
    let raw_value = entry
        .get_raw_otp_value()
        .ok_or_else(|| "Entry has no TOTP secret".to_string())?;
    let trimmed = raw_value.trim();

    if trimmed.is_empty() {
        return Err("Entry has no TOTP secret".to_string().into());
    }

    if raw {
        return Ok(trimmed.to_string().into());
    }

    let code = entry
        .get_otp()
        .map_err(|e| format!("Unable to read TOTP: {e}"))?
        .value_now()
        .map_err(|e| format!("Unable to compute TOTP: {e}"))?
        .code;

    Ok(code.into())
}
