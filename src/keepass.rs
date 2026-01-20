use std::{
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
};

use keepass::{
    Database, DatabaseKey,
    db::{Entry, Group, Node, Value},
    error::{DatabaseOpenError, DatabaseSaveError},
};

use crate::pwd::Pwd;

const MASKED_VALUE: &str = "******";

pub fn new_database_key(keyfile: Option<&Path>, password: Pwd) -> io::Result<DatabaseKey> {
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };
    let keyfile = read_file(keyfile)?;

    let mut key = DatabaseKey::new();

    key = match password {
        Some(password) => key.with_password(password),
        None => key,
    };
    if let Some(mut keyfile) = keyfile {
        key = key.with_keyfile(&mut keyfile)?;
    }

    Ok(key)
}

pub fn save_database(
    db: Database,
    dbfile: &Path,
    keyfile: Option<&Path>,
    password: Pwd,
) -> Result<(), DatabaseSaveError> {
    let key = new_database_key(keyfile, password)?;
    let mut file = File::create(dbfile)?;
    db.save(&mut file, key)?;
    Ok(())
}

pub fn open_database(
    password: Pwd,
    dbfile: &Path,
    keyfile: Option<&Path>,
) -> Result<Database, DatabaseOpenError> {
    let mut dbfile = read_file(Some(dbfile))?.expect("database path is always supplied");
    let key = new_database_key(keyfile, password)?;
    Database::open(&mut dbfile, key)
}

pub fn show_entry(entry: &Entry, show_sensitive: bool) -> String {
    let mut fields: Vec<String> = Vec::new();

    let standard_fields = [
        ("Title", entry.get_title()),
        ("Username", entry.get_username()),
        ("Password", entry.get_password()),
        ("URL", entry.get_url()),
        ("Notes", entry.get("Notes")),
    ];

    for (key, value) in standard_fields {
        if key == "Password" {
            if let Some(password) = value {
                if show_sensitive {
                    let trimmed_val = password.trim();
                    if !trimmed_val.is_empty() {
                        fields.push(format!("Password: {trimmed_val}"));
                    }
                } else {
                    fields.push(format!("Password: {MASKED_VALUE}"));
                }
            }
        } else if let Some(val) = value {
            let trimmed_val = val.trim();
            if !trimmed_val.is_empty() {
                fields.push(format!("{key}: {trimmed_val}"));
            }
        }
    }

    for (key, value) in entry.fields.iter() {
        if ["Title", "UserName", "URL", "Password", "Notes"].contains(&key.as_str()) {
            continue;
        }

        let value_str = match value {
            Value::Unprotected(s) => s.to_string(),
            Value::Protected(p) => {
                if show_sensitive {
                    String::from_utf8(p.unsecure().to_vec()).unwrap_or_default()
                } else {
                    MASKED_VALUE.to_string()
                }
            }
            Value::Bytes(b) => String::from_utf8(b.clone())
                .unwrap_or_else(|_| format!("<bytes: {}>", hex::encode(b))),
        };

        let trimmed_value = value_str.trim();
        if !trimmed_value.is_empty() {
            fields.push(format!("{key}: {trimmed_value}"));
        }
    }

    if let Some(code) = entry.get_otp().ok().and_then(|otp| otp.value_now().ok()) {
        fields.push(format!("TOTP Code: {}", code.code));
    }

    fields.join("\n")
}

fn read_file(file: Option<&Path>) -> io::Result<Option<Cursor<Vec<u8>>>> {
    if let Some(file) = file {
        let mut f = File::open(file)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(Some(Cursor::new(buf)))
    } else {
        Ok(None)
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
        self.entry
            .get_raw_otp_value()
            .map(|otp| !otp.trim().is_empty())
            .unwrap_or(false)
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
        assert_eq!(find_entry("My Other Title", &group), None);
    }
}
