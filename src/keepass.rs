use std::{
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
};

use keepass::{
    Database, DatabaseKey,
    db::{Entry, EntryId, EntryRef, GroupId, fields},
    error::{DatabaseOpenError, DatabaseSaveError},
};

use crate::pwd::Pwd;

const MASKED_VALUE: &str = "******";

pub fn new_database_key(keyfile: Option<&Path>, password: Pwd) -> io::Result<DatabaseKey> {
    let password = &password[..];
    let keyfile = read_file(keyfile)?;

    let mut key = DatabaseKey::new();
    key = key.with_password(password);

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

        let trimmed_value = value.get().trim();

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

/// Returns an iterator over all entries in the database, sorted by their path.
pub fn get_entries(db: &Database) -> impl Iterator<Item = EntryRef<'_>> {
    let mut ids = Vec::new();

    fn collect_entries(db: &Database, group_id: GroupId, ids: &mut Vec<EntryId>) {
        let Some(group) = db.group(group_id) else {
            return;
        };

        let mut entries = Vec::new();
        for entry in group.entries() {
            let title = entry.get(fields::TITLE).unwrap_or_default().to_string();
            entries.push((title, entry.id()));
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));
        ids.extend(entries.into_iter().map(|(_, id)| id));

        let mut groups = Vec::new();
        for subgroup in group.groups() {
            let name = subgroup.name.to_string();
            groups.push((name, subgroup.id()));
        }

        groups.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, subgroup_id) in groups {
            collect_entries(db, subgroup_id, ids);
        }
    }

    collect_entries(db, db.root().id(), &mut ids);

    ids.into_iter().filter_map(move |id| db.entry(id))
}

pub fn find_entry<'a>(query: &str, db: &'a Database) -> Option<EntryRef<'a>> {
    for entry in get_entries(db) {
        if entry.entry_path().ends_with(query) {
            return Some(entry);
        }
    }

    None
}

pub trait EntryPath {
    fn entry_path(&self) -> String;
}

impl EntryPath for EntryRef<'_> {
    fn entry_path(&self) -> String {
        let mut path = Vec::new();

        path.push(self.get_title().unwrap_or_default().to_string());

        let mut current = Some(self.parent().id());
        while let Some(group) = current.and_then(|id| self.database().group(id)) {
            path.push(group.name.to_string());
            current = group.parent().map(|p| p.id());
        }

        path.push("".to_string());

        path.reverse();
        path.join("/")
    }
}

#[cfg(test)]
mod tests {
    use keepass::db::fields;

    use super::*;

    #[test]
    fn test_find_entry() {
        let mut db = Database::new();

        let entry = db
            .root_mut()
            .edit(|g| g.name = "root".to_string())
            .add_group()
            .edit(|g| g.name = "child".to_string())
            .add_entry()
            .edit(|e| e.set_unprotected(fields::TITLE, "My Title".to_string()))
            .id();

        assert_eq!(
            db.entry(entry).unwrap().entry_path(),
            "/root/child/My Title"
        );

        assert!(find_entry("/root/child/My Title", &db).is_some());
        assert!(find_entry("child/My Title", &db).is_some());
        assert!(find_entry("My Title", &db).is_some());
        assert!(find_entry("Title", &db).is_some());
        assert!(find_entry("My Other Title", &db).is_none());
    }
}
