use std::{io, path::PathBuf};

use keepass::db::Entry;

use crate::{
    clipboard::set_clipboard,
    keepass::{find_entry, get_entries},
    pwd::Pwd,
    utils::{is_tty, open_database_interactively, skim},
    Result,
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

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
    #[arg(short, long, env = "KDBX_DATABASE")]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE")]
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
    );
    let db = db?;

    let query = args.entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let Some(entry) = find_entry(query, &db.root) {
            // Print totp to stdout when pipe used
            // e.g. `kdbx totp example.com | cat`
            if !is_tty(io::stdout()) {
                put!("{}", get_totp(entry, args.raw).to_string());
                return Ok(());
            }
            return clip(entry, args.raw);
        }
    }

    // If more than a single match has been found and stdout is not a TTY
    // than it is not possible to pick the right entry without user's interaction
    if !is_tty(io::stdout()) {
        return Err(format!("No single match for {}.", query.unwrap_or("[empty]")).into());
    }

    if let Some(wrapped_entry) = skim(
        &get_entries(&db.root, ""),
        query,
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
    if set_clipboard(Some(get_totp(entry, raw))).is_err() {
        return Err(format!(
            "Clipboard unavailable. Try use STDOUT, i.e. `kdbx totp '{}' | cat`.",
            entry.get_title().unwrap_or_default()
        )
        .into());
    }

    Ok(())
}

fn get_totp(entry: &Entry, raw: bool) -> Pwd {
    if raw {
        return entry
            .get_raw_otp_value()
            .unwrap_or_default()
            .to_string()
            .into();
    }
    entry
        .get_otp()
        .map(|v| v.value_now().map(|otpcode| otpcode.code).unwrap())
        .unwrap_or_default()
        .into()
}
