use crate::clipboard::set_clipboard;
use crate::keepass::open_database;
use crate::utils::{get_entries, is_tty, skim};
use crate::{Args, Result, CANCEL, CANCEL_RQ_FREQ};

use keepass::db::{Entry, NodeRef};
use log::*;

use std::io;
use std::thread;
use std::time;

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.flag_database.as_deref().unwrap().exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, _) = open_database(
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        args.flag_use_keyring,
    );
    let db = db?;

    let query = args.arg_entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let Some(NodeRef::Entry(entry)) = db.root.get(&[query]) {
            // Print password to stdout when pipe used
            // e.g. `kdbx clip example.com | cat`
            if !is_tty(io::stdout()) {
                put!("{} ", entry.get_password().unwrap_or_default());
                return Ok(());
            }
            return clip(entry, args.flag_timeout);
        }
    }

    // If more than a single match has been found and stdout is not a TTY
    // than it is not possible to pick the right entry without user's interaction
    if !is_tty(io::stdout()) {
        return Err(format!("No single match for {}.", query.unwrap_or("[empty]")).into());
    }

    if let Some(entry) = skim(
        &get_entries(&db.root, "".to_string()),
        query,
        args.flag_no_group,
        args.flag_preview,
        args.flag_full_screen,
    ) {
        clip(entry, args.flag_timeout)?
    }

    Ok(())
}

fn clip(entry: &Entry, timeout: Option<u8>) -> Result<()> {
    let pwd = entry.get_password().unwrap();

    if set_clipboard(Some(pwd.to_string())).is_err() {
        return Err(format!(
            "Clipboard unavailable. Try use STDOUT, i.e. `kdbx clip '{}' | cat`.",
            entry.get_title().unwrap_or_default()
        )
        .into());
    }

    if timeout.is_none() {
        debug!("user decided to leave the password in the buffer");
        return Ok(());
    }

    let mut ticks = u64::from(timeout.unwrap()) * CANCEL_RQ_FREQ;
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
