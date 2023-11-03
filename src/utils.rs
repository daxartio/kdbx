use std::{borrow::Cow, io, path::Path};

use keepass::{db::Entry, error::DatabaseOpenError, Database};
use log::*;
use skim::prelude::*;

use crate::{
    keepass::{open_database, show_entry, WrappedEntry},
    keyring::Keyring,
    pwd::Pwd,
    STDIN,
};

#[macro_export]
macro_rules! put {
    ($($arg:tt)*) => {
        use std::io::Write;
        let _ = write!(&mut ::std::io::stdout(), $($arg)*);
        let _ = ::std::io::stdout().flush();
    };
}

#[macro_export]
macro_rules! wout {
    ($($arg:tt)*) => ({
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stdout(), $($arg)*);
        let _ = ::std::io::stdout().flush();
    });
}

#[macro_export]
macro_rules! werr {
    ($($arg:tt)*) => ({
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($arg)*);
        let _ = ::std::io::stderr().flush();
    });
}

pub fn open_database_interactively(
    dbfile: &Path,
    keyfile: Option<&Path>,
    use_keyring: bool,
    remove_key: bool,
) -> (Result<Database, DatabaseOpenError>, Pwd) {
    if remove_key {
        if let Some(keyring) = Keyring::from_db_path(dbfile) {
            if let Err(msg) = keyring.delete_password() {
                werr!("No key removed for `{}`. {}", dbfile.to_string_lossy(), msg);
            }
        }
    }

    let keyring = if use_keyring {
        Keyring::from_db_path(dbfile).map(|k| {
            debug!("keyring: {}", k);
            k
        })
    } else {
        None
    };

    if let Some(Ok(password)) = keyring.as_ref().map(|k| k.get_password()) {
        if let Ok(db) = open_database(password.clone(), dbfile, keyfile) {
            return (Ok(db), password);
        }

        warn!("removing wrong password in the keyring");
        let _ = keyring.as_ref().map(|k| k.delete_password());
    }

    // Try read password from pipe
    if !is_tty(io::stdin()) {
        let password = STDIN.read_password();
        return (open_database(password.clone(), dbfile, keyfile), password);
    }

    // Allow multiple attempts to enter the password from TTY
    let mut att: u8 = 3;
    loop {
        put!("Password: ");

        let password = STDIN.read_password();
        let db = open_database(password.clone(), dbfile, keyfile);

        // If opened successfully store the password
        if db.is_ok() {
            let _ = keyring.as_ref().map(|k| k.set_password(&password));
        }

        att -= 1;

        if db.is_ok() || att == 0 {
            break (db.map_err(From::from), password);
        }

        wout!("{} attempt(s) left.", att);
    }
}

struct EntryItem {
    idx: usize,
    title: String,
    props: Option<String>,
}

pub fn skim<'a>(
    entries: &[WrappedEntry<'a>],
    query: Option<&str>,
    hide_groups: bool,
    show_preview: bool,
    full_screen: bool,
    with_totp: bool,
) -> Option<&'a Entry> {
    let opts = SkimOptionsBuilder::default()
        .multi(false)
        .reverse(true)
        .query(query)
        .header(if full_screen { Some(" ") } else { None }) // separate counters and entries with a line
        .color(Some("16")) // 16 colors scheme
        .height(Some("7"))
        .no_height(full_screen)
        .bind(vec![
            "ctrl-q:ignore", // toggle interactive
            "ctrl-l:ignore", // clear screen
            "ctrl-r:ignore", // rotate mode
        ])
        .delimiter(if hide_groups { None } else { Some("/") })
        .preview(if show_preview { Some("") } else { None })
        .preview_window(Some("right:65%"))
        .build()
        .expect("well formed SkimOptions");

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let entries = entries
        .iter()
        .filter(|e| {
            if with_totp {
                e.entry.get_raw_otp_value().is_some()
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    entries
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let title = if hide_groups {
                e.entry.get_title().unwrap_or_default().to_owned()
            } else {
                e.entry_path()
            };

            let props = if show_preview {
                Some(show_entry(e.entry))
            } else {
                None
            };

            EntryItem { idx, title, props }
        })
        .for_each(|item| tx.send(Arc::new(item)).unwrap());

    // No more input expected, dropping sender
    drop(tx);

    Skim::run_with(&opts, Some(rx))
        .map(|res| {
            if res.is_abort || res.selected_items.len() != 1 {
                None
            } else {
                res.selected_items[0]
                    .as_ref()
                    .as_any()
                    .downcast_ref::<EntryItem>()
                    .map(|ei: &EntryItem| entries[ei.idx].entry)
            }
        })
        .unwrap()
}

impl SkimItem for EntryItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.title)
    }

    fn preview(&self, _: PreviewContext) -> ItemPreview {
        if let Some(props) = &self.props {
            ItemPreview::Text(props.to_owned())
        } else {
            ItemPreview::Global
        }
    }
}

pub fn is_tty(fd: impl std::os::unix::io::AsRawFd) -> bool {
    unsafe { ::libc::isatty(fd.as_raw_fd()) == 1 }
}
