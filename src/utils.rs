use keepass::db::{Entry, Group, Node};
use skim::prelude::*;

use std::borrow::Cow;

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

pub fn show_entry(entry: &Entry) -> String {
    format!(
        "Title: {}\nUsername: {}\nUrl: {}\nNote: {}",
        entry.get_title().unwrap_or_default(),
        entry.get_username().unwrap_or_default(),
        entry.get_url().unwrap_or_default(),
        entry.get("Notes").unwrap_or_default(),
    )
}

pub fn is_tty(fd: impl std::os::unix::io::AsRawFd) -> bool {
    unsafe { ::libc::isatty(fd.as_raw_fd()) == 1 }
}

pub fn get_entries(group: &Group, path: String) -> Vec<WrappedEntry<'_>> {
    let mut entries = vec![];
    group.children.iter().for_each(|v| match v {
        Node::Entry(entry) => entries.push(WrappedEntry {
            path: format!("{}/{}", path, group.name),
            entry,
        }),
        Node::Group(child) => {
            if !child.children.is_empty() {
                entries.extend(get_entries(child, format!("{}/{}", path, group.name)))
            }
        }
    });
    entries
}

pub struct WrappedEntry<'a> {
    pub path: String,
    pub entry: &'a Entry,
}

impl WrappedEntry<'_> {
    pub fn entry_path(&self) -> String {
        format!(
            "{}/{}",
            self.path,
            self.entry.get_title().unwrap_or_default().to_owned(),
        )
    }
}

pub fn find_entry<'a>(query: &str, group: &'a Group) -> Option<&'a Entry> {
    get_entries(group, "".to_string()).iter().find_map(|e| {
        let entry_path = e.entry_path();
        if entry_path.ends_with(query) {
            Some(e.entry)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use keepass::db::Value;

    use super::*;

    #[test]
    fn test_find_entry() {
        let mut group = Group::new("root");
        let mut child = Group::new("child");
        let mut entry = Entry::new();
        entry.fields.insert(
            "Title".to_string(),
            Value::Unprotected("My Title".to_string()),
        );

        child.children.push(Node::Entry(entry.clone()));
        group.children.push(Node::Group(child));

        assert_eq!(find_entry("/root/child/My Title", &group), Some(&entry));
        assert_eq!(find_entry("child/My Title", &group), Some(&entry));
        assert_eq!(find_entry("My Title", &group), Some(&entry));
        assert_eq!(find_entry("Title", &group), Some(&entry));
        assert!(find_entry("My Other Title", &group).is_none());
    }
}
