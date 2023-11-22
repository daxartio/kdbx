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

pub fn new_database_key(keyfile: Option<&Path>, password: Pwd) -> DatabaseKey {
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };
    let keyfile = read_file(keyfile);

    let key = DatabaseKey::new();

    let key = match password {
        Some(password) => key.with_password(password),
        None => key,
    };
    match keyfile {
        Some(mut keyfile) => key.with_keyfile(&mut keyfile).unwrap(),
        None => key,
    }
}

pub fn save_database(db: Database, dbfile: &Path, keyfile: Option<&Path>, password: Pwd) {
    let key = new_database_key(keyfile, password);
    db.save(&mut File::create(dbfile).unwrap(), key).unwrap();
}

pub fn open_database(
    password: Pwd,
    dbfile: &Path,
    keyfile: Option<&Path>,
) -> Result<Database, DatabaseOpenError> {
    let mut dbfile = read_file(Some(dbfile));
    let key = new_database_key(keyfile, password);
    Database::open(dbfile.as_mut().map(|f| f as &mut dyn Read).unwrap(), key)
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

pub fn get_entries(group: &Group, path: impl ToString) -> Vec<WrappedEntry<'_>> {
    let mut entries = Vec::with_capacity(
        group
            .children
            .iter()
            .filter(|v| match v {
                Node::Entry(_) => true,
                Node::Group(_) => false,
            })
            .count(),
    );
    group.children.iter().for_each(|v| match v {
        Node::Entry(entry) => entries.push(WrappedEntry {
            path: format!("{}/{}", path.to_string(), group.name),
            entry,
        }),
        Node::Group(child) => {
            if !child.children.is_empty() {
                entries.extend(get_entries(
                    child,
                    format!("{}/{}", path.to_string(), group.name),
                ))
            }
        }
    });
    entries
}

pub struct WrappedEntry<'a> {
    pub path: String,
    pub entry: &'a Entry,
}

impl EntryPath for WrappedEntry<'_> {
    fn entry_path(&self) -> String {
        format!(
            "{}/{}",
            self.path,
            self.entry.get_title().unwrap_or_default().to_owned(),
        )
    }

    fn get_entry(&self) -> &Entry {
        self.entry
    }

    fn get_title(&self) -> String {
        self.entry.get_title().unwrap_or_default().to_owned()
    }

    fn has_totp(&self) -> bool {
        self.entry.get_raw_otp_value().is_some()
    }
}

pub fn find_entry<'a>(query: &str, group: &'a Group) -> Option<&'a Entry> {
    get_entries(group, "").iter().find_map(|e| {
        let entry_path = e.entry_path();
        if entry_path.ends_with(query) {
            Some(e.entry)
        } else {
            None
        }
    })
}

pub trait EntryPath {
    fn entry_path(&self) -> String;
    fn get_entry(&self) -> &Entry;
    fn get_title(&self) -> String;
    fn has_totp(&self) -> bool;
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
