use std::{
    borrow::Cow,
    error, fmt,
    fs::File,
    io::{self, Read},
    ops::Deref,
    path::Path,
};

use keepass::{
    Database,
    db::{EntryRef, fields},
    error::DatabaseOpenError as KeepassOpenError,
};
use log::*;
use regex::Regex;
use skim::{prelude::*, tui::options::PreviewLayout};

use crate::{
    STDIN,
    keepass::{EntryPath, open_database, show_entry},
    keyring::Keyring,
    pwd::Pwd,
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

#[derive(Debug)]
pub enum DatabaseOpenError {
    NotADatabase,
    NoInteraction,
    KeepassOpenError(KeepassOpenError),
}

impl fmt::Display for DatabaseOpenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DatabaseOpenError::NotADatabase => write!(f, "Invalid file"),
            DatabaseOpenError::NoInteraction => write!(f, "No interaction"),
            DatabaseOpenError::KeepassOpenError(..) => write!(f, "Invalid password or key"),
        }
    }
}

impl error::Error for DatabaseOpenError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DatabaseOpenError::NotADatabase => None,
            DatabaseOpenError::NoInteraction => None,
            DatabaseOpenError::KeepassOpenError(ref e) => Some(e),
        }
    }
}

impl From<KeepassOpenError> for DatabaseOpenError {
    fn from(value: KeepassOpenError) -> Self {
        DatabaseOpenError::KeepassOpenError(value)
    }
}

fn verify_database_signature(dbfile: &Path) -> Result<bool, Box<dyn error::Error>> {
    // Read the first 8 bytes that will contain the correct signature if the given file is a KeePass database
    let mut file = File::open(dbfile)?;
    let mut signature_bytes = [0; 8];
    file.read_exact(&mut signature_bytes)?;
    let signature = u64::from_le_bytes(signature_bytes);

    // https://keepass.info/help/kb/kdbx.html
    let correct_signature_bytes: [u8; 8] = hex::decode("B54BFB679AA2D903")?
        .try_into()
        .map_err(|_| "")?;
    let correct_signature = u64::from_be_bytes(correct_signature_bytes);

    Ok(signature == correct_signature)
}

pub fn open_database_interactively(
    dbfile: &Path,
    keyfile: Option<&Path>,
    use_keyring: bool,
    remove_key: bool,
    no_interaction: bool,
) -> Result<(Database, Pwd), DatabaseOpenError> {
    if !verify_database_signature(dbfile).map_err(|_| DatabaseOpenError::NotADatabase)? {
        return Err(DatabaseOpenError::NotADatabase);
    }

    if remove_key
        && let Some(keyring) = Keyring::from_db_path(dbfile)
        && let Err(msg) = keyring.delete_password()
    {
        werr!("No key removed for `{}`. {}", dbfile.to_string_lossy(), msg);
    }

    let keyring = if use_keyring {
        Keyring::from_db_path(dbfile).map(|k| {
            debug!("keyring: {k}");
            k
        })
    } else {
        None
    };

    if let Some(Ok(password)) = keyring.as_ref().map(|k| k.get_password()) {
        if let Ok(db) = open_database(password.clone(), dbfile, keyfile) {
            return Ok((db, password));
        }

        warn!("removing wrong password in the keyring");
        let _ = keyring.as_ref().map(|k| k.delete_password());
    }

    // Try read password from pipe
    if !is_tty(io::stdin()) {
        let password = STDIN.read_password();
        return Ok((open_database(password.clone(), dbfile, keyfile)?, password));
    }

    if no_interaction {
        return Err(DatabaseOpenError::NoInteraction);
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
            break Ok((db?, password));
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
    entries: Vec<EntryRef<'a>>,
    query: Option<String>,
    hide_groups: bool,
    show_preview: bool,
    full_screen: bool,
    with_totp: bool,
) -> Option<EntryRef<'a>> {
    let opts = SkimOptionsBuilder::default()
        .multi(false)
        .reverse(true)
        .query(query)
        .header(if full_screen {
            Some(" ".to_string())
        } else {
            None
        }) // separate counters and entries with a line
        .color(Some("16".to_string())) // 16 colors scheme
        .height("7".to_string())
        .no_height(full_screen)
        .bind(vec![
            "ctrl-q:ignore".to_string(), // toggle interactive
            "ctrl-l:ignore".to_string(), // clear screen
            "ctrl-r:ignore".to_string(), // rotate mode
        ])
        .delimiter(if hide_groups {
            Regex::new(r"$^").expect("valid regex")
        } else {
            Regex::new("/").expect("valid regex")
        })
        .preview(if show_preview {
            Some("".to_string())
        } else {
            None
        })
        .preview_window(PreviewLayout::from("right:65%"))
        .build()
        .expect("well formed SkimOptions");

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let mut entries = entries
        .into_iter()
        .filter(|e| {
            if with_totp {
                e.get_raw_otp_value().is_some()
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
                e.get(fields::TITLE).unwrap_or_default().to_string()
            } else {
                e.entry_path()
            };

            let props = if show_preview {
                Some(show_entry(e.deref(), false))
            } else {
                None
            };

            EntryItem { idx, title, props }
        })
        .for_each(|item| tx.send(Arc::new(item)).unwrap());

    // No more input expected, dropping sender
    drop(tx);

    let res = Skim::run_with(opts, Some(rx))
        .map(|res| {
            if res.is_abort || res.selected_items.len() != 1 {
                None
            } else {
                res.selected_items[0]
                    .item
                    .as_ref()
                    .as_any()
                    .downcast_ref::<EntryItem>()
                    .map(|ei: &EntryItem| ei.idx)
            }
        })
        .unwrap();

    if let Some(idx) = res {
        Some(entries.remove(idx))
    } else {
        None
    }
}

impl SkimItem for EntryItem {
    fn text(&self) -> Cow<'_, str> {
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
