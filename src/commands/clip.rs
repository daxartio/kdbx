use crate::clipboard::set_clipboard;
use crate::utils::{is_tty, open_database, skim};
use crate::{Args, Result, CANCEL, CANCEL_RQ_FREQ};

use log::*;

use std::io;
use std::thread;
use std::time;

pub(crate) fn run(args: Args) -> Result<()> {
    let db = open_database(
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        args.flag_use_keyring,
    )?;

    let query = args.arg_entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let [entry] = db.find(query).as_slice() {
            // Print password to stdout when pipe used
            // e.g. `kdbx clip example.com | cat`
            if !is_tty(io::stdout()) {
                put!("{} ", entry.password()?);
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
        &db.entries(),
        query,
        args.flag_no_group,
        args.flag_preview,
        args.flag_full_screen,
    ) {
        clip(entry, args.flag_timeout)?
    }

    Ok(())
}

fn clip<'a>(entry: &'a kdbx4::Entry<'a>, timeout: Option<u8>) -> Result<()> {
    let pwd = entry.password()?;

    if set_clipboard(Some(pwd)).is_err() {
        return Err(format!(
            "Clipboard unavailable. Try use STDOUT, i.e. `kdbx clip '{}' | cat`.",
            entry.title()
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
