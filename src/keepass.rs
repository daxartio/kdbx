use std::{
    error, fmt,
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
    let mut lines: Vec<String> = Vec::new();

    let standard_fields = [
        (fields::TITLE, "Title"),
        (fields::USERNAME, "Username"),
        (fields::PASSWORD, "Password"),
        (fields::URL, "URL"),
        (fields::NOTES, "Notes"),
    ];

    for (key, label) in standard_fields {
        push_field(&mut lines, entry, key, label, show_sensitive);
    }

    for (key, value) in entry.fields.iter() {
        if fields::KNOWN_FIELDS.contains(&key.as_str()) {
            continue;
        }

        if key == fields::OTP {
            push_field(&mut lines, entry, key, key, show_sensitive);
            continue;
        }

        let trimmed_value = value.get().trim();
        if trimmed_value.is_empty() {
            continue;
        }

        if value.is_protected() && !show_sensitive {
            lines.push(format!("{key}: {MASKED_VALUE}"));
        } else {
            lines.push(format!("{key}: {trimmed_value}"));
        }
    }

    if show_sensitive && let Some(code) = entry.get_otp().ok().and_then(|otp| otp.value_now().ok())
    {
        lines.push(format!("TOTP Code: {}", code.code));
    }

    lines.join("\n")
}

fn push_field(
    lines: &mut Vec<String>,
    entry: &Entry,
    key: &str,
    label: &str,
    show_sensitive: bool,
) {
    let Some(value) = entry.fields.get(key) else {
        return;
    };

    let trimmed_value = value.get().trim();
    if trimmed_value.is_empty() {
        return;
    }

    if (key == fields::PASSWORD || key == fields::OTP || value.is_protected()) && !show_sensitive {
        lines.push(format!("{label}: {MASKED_VALUE}"));
    } else {
        lines.push(format!("{label}: {trimmed_value}"));
    }
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

#[derive(Debug, PartialEq, Eq)]
pub enum EntryLookupError {
    Ambiguous { query: String, matches: usize },
}

impl fmt::Display for EntryLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryLookupError::Ambiguous { query, matches } => {
                write!(f, "Ambiguous entry `{query}` ({matches} matches)")
            }
        }
    }
}

impl error::Error for EntryLookupError {}

pub fn find_entry<'a>(
    query: &str,
    db: &'a Database,
) -> Result<Option<EntryRef<'a>>, EntryLookupError> {
    let mut matches = get_entries(db).filter(|entry| is_entry_match(entry, query));
    let first = matches.next();
    let second = matches.next();

    if second.is_some() {
        return Err(EntryLookupError::Ambiguous {
            query: query.to_string(),
            matches: 2 + matches.count(),
        });
    }

    Ok(first)
}

fn is_entry_match(entry: &EntryRef<'_>, query: &str) -> bool {
    let path = entry.entry_path();

    entry.get_title() == Some(query)
        || path == query
        || path
            .strip_suffix(query)
            .is_some_and(|prefix| prefix.ends_with('/'))
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

        assert!(find_entry("/root/child/My Title", &db).unwrap().is_some());
        assert!(find_entry("child/My Title", &db).unwrap().is_some());
        assert!(find_entry("My Title", &db).unwrap().is_some());
        assert!(find_entry("Title", &db).unwrap().is_none());
        assert!(find_entry("My Other Title", &db).unwrap().is_none());
    }

    #[test]
    fn test_find_entry_ambiguous() {
        let mut db = Database::new();
        db.root_mut().edit(|g| g.name = "root".to_string());

        db.root_mut()
            .add_group()
            .edit(|g| g.name = "first".to_string())
            .add_entry()
            .edit(|e| e.set_unprotected(fields::TITLE, "Shared".to_string()));

        db.root_mut()
            .add_group()
            .edit(|g| g.name = "second".to_string())
            .add_entry()
            .edit(|e| e.set_unprotected(fields::TITLE, "Shared".to_string()));

        assert!(matches!(
            find_entry("Shared", &db),
            Err(EntryLookupError::Ambiguous { matches: 2, .. })
        ));
        assert!(find_entry("first/Shared", &db).unwrap().is_some());
    }

    #[test]
    fn test_show_entry_masks_protected_fields() {
        let mut db = Database::new();

        let entry = db
            .root_mut()
            .add_entry()
            .edit(|e| {
                e.set_unprotected(fields::TITLE, "TOTP Entry".to_string());
                e.set_protected(fields::PASSWORD, "password".to_string());
                e.set_unprotected(
                    fields::OTP,
                    "otpauth://totp/TOTP%20Entry:user?secret=JBSWY3DPEHPK3PXP&period=30&digits=6&\
                     issuer=TOTP%20Entry"
                        .to_string(),
                );
                e.set_protected("api_key", "secret-token".to_string());
            })
            .id();

        let output = show_entry(&db.entry(entry).unwrap(), false);

        assert!(output.contains("Password: ******"));
        assert!(output.contains("otp: ******"));
        assert!(output.contains("api_key: ******"));
        assert!(!output.contains("secret-token"));
        assert!(!output.contains("JBSWY3DPEHPK3PXP"));
        assert!(!output.contains("TOTP Code:"));
    }

    #[test]
    fn test_show_entry_reveals_sensitive_fields_when_requested() {
        let mut db = Database::new();

        let entry = db
            .root_mut()
            .add_entry()
            .edit(|e| {
                e.set_unprotected(fields::TITLE, "TOTP Entry".to_string());
                e.set_protected(fields::PASSWORD, "password".to_string());
                e.set_unprotected(
                    fields::OTP,
                    "otpauth://totp/TOTP%20Entry:user?secret=JBSWY3DPEHPK3PXP&period=30&digits=6&\
                     issuer=TOTP%20Entry"
                        .to_string(),
                );
                e.set_protected("api_key", "secret-token".to_string());
            })
            .id();

        let output = show_entry(&db.entry(entry).unwrap(), true);

        assert!(output.contains("Password: password"));
        assert!(output.contains("otp: otpauth://"));
        assert!(output.contains("api_key: secret-token"));
        assert!(output.contains("TOTP Code:"));
    }
}
