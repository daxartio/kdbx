use std::{io, path::PathBuf, thread, time};

use clap::ValueHint;
use keepass::db::{EntryRef, History, fields};
use log::*;

use crate::{
    CANCEL, CANCEL_RQ_FREQ, Result, STDIN,
    clipboard::set_clipboard,
    keepass::{find_entry, get_entries, save_database},
    pwd::Pwd,
    utils::{DatabaseOpenResult, is_tty, open_database_interactively, skim},
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Timeout in seconds before clearing the clipboard. 0 means no clean-up
    #[arg(short, long, default_value_t = crate::DEFAULT_TIMEOUT)]
    timeout: u8,

    /// Change the entry password, preserving the previous version in history
    #[arg(short = 'e', long)]
    edit: bool,

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

enum PwdMode {
    Show,
    Edit,
}

impl From<&Args> for PwdMode {
    fn from(args: &Args) -> Self {
        if args.edit { Self::Edit } else { Self::Show }
    }
}

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.database.exists() {
        return Err("File does not exist".to_string().into());
    }
    let DatabaseOpenResult::Opened(mut db, password) = open_database_interactively(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
        args.no_interaction,
    )?
    else {
        return Ok(());
    };

    let mode: PwdMode = (&args).into();

    let query = args.entry.as_ref().map(String::as_ref);

    let entry = if let Some(entry) = query
        .map(|query| find_entry(query, &db))
        .transpose()?
        .flatten()
    {
        entry
    } else {
        if args.no_interaction {
            return Err("Not found".to_string().into());
        }

        if !is_tty(io::stdout()) {
            return Err(format!("No single match for {}.", query.unwrap_or("[empty]")).into());
        }

        let Some(entry) = skim(
            get_entries(&db).collect::<Vec<_>>(),
            query.map(String::from),
            args.no_group,
            args.preview,
            args.full_screen,
            false,
        ) else {
            return Ok(());
        };
        entry
    };

    match mode {
        PwdMode::Show => {
            if !is_tty(io::stdout()) {
                put!("{}", entry.get_password().unwrap_or_default());
            } else {
                clip(&entry, args.timeout)?;
            }
        }
        PwdMode::Edit => {
            let interactive = is_tty(io::stdin());
            if interactive && args.no_interaction {
                return Err("No interaction".into());
            }
            if interactive {
                put!("New password: ");
            }
            let new_password = STDIN.try_read_password()?;
            if interactive {
                put!("Confirm: ");
            }
            let confirm = STDIN.try_read_password()?;
            if new_password != confirm {
                return Err("Passwords do not match".into());
            }
            if entry.get_password() == Some(new_password.as_ref()) {
                return Ok(());
            }

            let entry_id = entry.id();
            {
                let mut entry = db.entry_mut(entry_id).ok_or("Entry no longer exists")?;
                entry
                    .track_changes()
                    .set_protected(fields::PASSWORD, new_password.as_ref());
                // keepass 0.17 prepends snapshots; KeePass expects oldest-first history.
                if let Some(history) = entry.history.take() {
                    let versions = history.get_entries();
                    let mut ordered = History::default();
                    for version in versions.iter().take(1).chain(versions.iter().skip(1).rev()) {
                        ordered.add_entry(version.clone());
                    }
                    entry.history = Some(ordered);
                }
            }
            save_database(*db, &args.database, args.key_file.as_deref(), password)?;
        }
    }

    Ok(())
}

fn clip(entry: &EntryRef<'_>, timeout: u8) -> Result<()> {
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
        if ticks.is_multiple_of(CANCEL_RQ_FREQ) {
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
