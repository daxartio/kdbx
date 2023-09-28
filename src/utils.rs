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

    entries
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let title = if hide_groups {
                e.entry.get_title().unwrap_or_default().to_owned()
            } else {
                format!(
                    "{}/{}",
                    e.path,
                    e.entry.get_title().unwrap_or_default().to_owned(),
                )
            };

            let props = if show_preview {
                Some(format!(
                    "{} {}",
                    e.entry.get_title().unwrap_or_default(),
                    e.entry.get_username().unwrap_or_default()
                ))
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
    path: String,
    entry: &'a Entry,
}
