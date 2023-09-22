use crate::keyring::Keyring;
use crate::Result;
use crate::STDIN;

use kdbx4::{CompositeKey, Database, Entry, Kdbx4};
use skim::prelude::*;

use log::*;

use std::borrow::Cow;
use std::io;
use std::path::Path;

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

pub fn open_database(dbfile: &Path, keyfile: Option<&Path>, use_keyring: bool) -> Result<Database> {
    let keyring = if use_keyring {
        Keyring::from_db_path(dbfile).map(|k| {
            debug!("keyring: {}", k);
            k
        })
    } else {
        None
    };

    // Try to open DB with a key from keyring
    if let Some(Ok(pwd)) = keyring.as_ref().map(|k| k.get_password()) {
        let key = CompositeKey::new(Some(&pwd), keyfile)?;
        if let Ok(db) = Kdbx4::open(dbfile, key) {
            return Ok(db);
        }

        warn!("removing wrong password in the keyring");
        let _ = keyring.as_ref().map(|k| k.delete_password());
    }

    // Try read password from pipe
    if !is_tty(io::stdin()) {
        let pwd = STDIN.read_password();
        let key = CompositeKey::new(Some(&pwd), keyfile)?;
        let db = Kdbx4::open(dbfile, key)?;
        return Ok(db);
    }

    // Allow multiple attempts to enter the password from TTY
    let mut att = 3;
    loop {
        put!("Password:");

        let pwd = STDIN.read_password();
        let key = CompositeKey::new(Some(&pwd), keyfile)?;
        let db = Kdbx4::open(dbfile, key);

        // If opened successfully store the password
        if db.is_ok() {
            let _ = keyring.as_ref().map(|k| k.set_password(&pwd));
        }

        att -= 1;

        if db.is_ok() || att == 0 {
            break db.map_err(From::from);
        }

        wout!("{} attempt(s) left.", att);
    }
}

pub fn skim<'a>(
    entries: &'a [Entry<'a>],
    query: Option<&'a str>,
    hide_groups: bool,
    show_preview: bool,
    full_screen: bool,
) -> Option<&'a Entry<'a>> {
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

    entries
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let title = if hide_groups {
                e.title().to_owned()
            } else {
                format!("/{}/{}", e.group(), e.title())
            };

            let props = if show_preview {
                Some(format!("{}", e))
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
                    .map(|ei| &entries[ei.idx])
            }
        })
        .unwrap()
}

struct EntryItem {
    idx: usize,
    title: String,
    props: Option<String>,
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
