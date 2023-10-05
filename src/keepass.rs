use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

use keepass::{
    db::{Entry, Group, Node},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use crate::pwd::Pwd;

pub fn save_database(db: Database, dbfile: &Path, keyfile: Option<&Path>, password: Pwd) {
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };
    let mut keyfile = read_file(keyfile);
    db.save(
        &mut File::create(dbfile).unwrap(),
        DatabaseKey {
            password,
            keyfile: keyfile.as_mut().map(|f| f as &mut dyn Read),
        },
    )
    .unwrap();
}

pub fn open_database(
    password: Pwd,
    dbfile: &Path,
    keyfile: Option<&Path>,
) -> Result<Database, DatabaseOpenError> {
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };

    let mut dbfile = read_file(Some(dbfile));
    let mut keyfile = read_file(keyfile);

    Database::open(
        dbfile.as_mut().map(|f| f as &mut dyn Read).unwrap(),
        DatabaseKey {
            password,
            keyfile: keyfile.as_mut().map(|f| f as &mut dyn Read),
        },
    )
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

fn read_file(file: Option<&Path>) -> Option<Cursor<Vec<u8>>> {
    if let Some(file) = file {
        let mut f = File::open(file).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        Some(Cursor::new(buf))
    } else {
        None
    }
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
