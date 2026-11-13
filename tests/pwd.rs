use std::{
    fs::{self, File},
    path::Path,
};

use assert_cmd::{assert::Assert, cargo::cargo_bin_cmd};
use keepass::{
    Database, DatabaseKey,
    config::{DatabaseConfig, DatabaseVersion},
    db::{Entry, GroupRef, fields},
};
use rstest::rstest;
use tempfile::TempDir;

fn test_database_key() -> DatabaseKey {
    let mut keyfile = File::open("tests/files/secret").unwrap();
    DatabaseKey::new()
        .with_password("test123")
        .with_keyfile(&mut keyfile)
        .unwrap()
}

fn open_test_database(path: &Path) -> Database {
    Database::open(&mut File::open(path).unwrap(), test_database_key()).unwrap()
}

fn create_editable_database(path: &Path) {
    let mut db = open_test_database(Path::new("tests/files/test.kdbx"));
    db.config.version = DatabaseConfig::default().version;
    db.save(&mut File::create(path).unwrap(), test_database_key())
        .unwrap();
}

fn all_entries(group: GroupRef<'_>) -> Vec<Entry> {
    let mut entries: Vec<_> = group.entries().map(|entry| entry.clone()).collect();
    for child in group.groups() {
        entries.extend(all_entries(child));
    }
    entries
}

fn password_entry(db: &Database) -> Entry {
    all_entries(db.root())
        .into_iter()
        .find(|entry| entry.get_title() == Some("test-pwd"))
        .unwrap()
}

fn edit_password(path: &Path, input: &str) -> Assert {
    cargo_bin_cmd!()
        .args(["pwd", "--edit", "--no-interaction", "-d"])
        .arg(path)
        .args(["-k", "tests/files/secret", "test-pwd"])
        .write_stdin(input)
        .assert()
}

#[test]
fn test_pwd() {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd
        .args([
            "pwd",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
            "test-pwd",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd.success().stdout("1234");
}

#[test]
fn test_pwd_empty_master_password_from_empty_stdin() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    let mut db = open_test_database(Path::new("tests/files/test.kdbx"));
    db.config.version = DatabaseConfig::default().version;
    db.save(
        &mut File::create(&path).unwrap(),
        DatabaseKey::new().with_password(""),
    )
    .unwrap();
    cargo_bin_cmd!()
        .env_remove("KDBX_KEY_FILE")
        .args(["pwd", "-n", "-d"])
        .arg(&path)
        .arg("test-pwd")
        .write_stdin("")
        .assert()
        .success()
        .stdout("1234")
        .stderr("");
}

#[test]
fn test_pwd_no_interaction_not_found() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "pwd",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "unknown-pwd",
        "-n",
    ])
    .write_stdin("test123")
    .assert()
    .stderr("Not found\n");
}

#[test]
fn test_pwd_no_interaction_open_error() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "pwd",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "unknown-pwd",
        "-n",
    ])
    .write_stdin("123")
    .assert()
    .stderr("Invalid password or key\n");
}

#[test]
fn test_pwd_edit_preserves_entry_history() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    create_editable_database(&path);
    let original_db = open_test_database(&path);
    let original = password_entry(&original_db);
    let original_history = original
        .history
        .as_ref()
        .map_or(0, |history| history.get_entries().len());

    for (revision, new_password) in ["replacement-dummy-password", "second-dummy-password"]
        .iter()
        .enumerate()
    {
        let previous = password_entry(&open_test_database(&path));
        edit_password(&path, &format!("test123\n{new_password}\n{new_password}\n"))
            .success()
            .stdout("")
            .stderr("");

        let updated_db = open_test_database(&path);
        let updated = password_entry(&updated_db);
        assert_eq!(updated.get_password(), Some(*new_password));
        assert!(updated.fields.get(fields::PASSWORD).unwrap().is_protected());
        assert_eq!(updated.id(), original.id());
        assert!(updated.times.last_modification > original.times.last_modification);
        let history = updated.history.as_ref().unwrap();
        assert_eq!(history.get_entries().len(), original_history + revision + 1);
        let historical = history.get_entries().last().unwrap();
        let previous_history = previous.history.as_ref().unwrap().get_entries();
        assert_eq!(
            &history.get_entries()[..previous_history.len()],
            previous_history
        );
        assert_eq!(historical.fields, previous.fields);
        assert_eq!(historical.times, previous.times);
        assert_eq!(historical.id(), previous.id());
        assert!(historical.history.is_none());
        for original_entry in all_entries(original_db.root()) {
            if original_entry.id() != original.id() {
                let updated_entry = updated_db.entry(original_entry.id()).unwrap();
                assert_eq!(*updated_entry, original_entry);
            }
        }
    }

    let before = fs::read(&path).unwrap();
    edit_password(
        &path,
        "test123\nsecond-dummy-password\nsecond-dummy-password\n",
    )
    .success()
    .stdout("")
    .stderr("");
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn test_pwd_edit_kdbx4_0_preserves_history_and_encryption_settings() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    fs::copy("tests/files/test.kdbx", &path).unwrap();
    let original_db = open_test_database(&path);
    assert_eq!(original_db.config.version, DatabaseVersion::KDB4(0));
    let original = password_entry(&original_db);
    let mut expected_config = original_db.config.clone();
    expected_config.version = DatabaseVersion::KDB4(1);

    edit_password(&path, "test123\nnew-dummy-password\nnew-dummy-password\n")
        .success()
        .stdout("")
        .stderr("");

    let updated_db = open_test_database(&path);
    assert_eq!(updated_db.config, expected_config);
    let updated = updated_db.entry(original.id()).unwrap();
    assert_eq!(updated.get_password(), Some("new-dummy-password"));
    let historical = updated
        .history
        .as_ref()
        .unwrap()
        .get_entries()
        .last()
        .unwrap();
    let mut expected_historical = original.clone();
    expected_historical.history = None;
    assert_eq!(*historical, expected_historical);
    for original_entry in all_entries(original_db.root()) {
        if original_entry.id() != original.id() {
            assert_eq!(
                *updated_db.entry(original_entry.id()).unwrap(),
                original_entry
            );
        }
    }
}

#[rstest]
#[case(
    "test123\nnew-dummy-password\ndifferent-dummy-password\n",
    "Passwords do not match\n"
)]
#[case("test123\n", "Password input ended\n")]
#[case("test123\nnew-dummy-password\n", "Password input ended\n")]
fn test_pwd_edit_invalid_input_does_not_modify_database(
    #[case] input: &str,
    #[case] error: &'static str,
) {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    create_editable_database(&path);
    let before = fs::read(&path).unwrap();
    edit_password(&path, input)
        .failure()
        .stdout("")
        .stderr(error);
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[rstest]
#[case("new-dummy-password")]
#[case("")]
fn test_pwd_edit_without_keyfile_or_history(#[case] new_password: &str) {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    let mut db = open_test_database(Path::new("tests/files/test.kdbx"));
    db.config.version = DatabaseConfig::default().version;
    let original = password_entry(&db);
    db.entry_mut(original.id()).unwrap().history = None;
    db.save(
        &mut File::create(&path).unwrap(),
        DatabaseKey::new().with_password("test123"),
    )
    .unwrap();
    cargo_bin_cmd!()
        .env_remove("KDBX_KEY_FILE")
        .args(["pwd", "-e", "-n", "-d"])
        .arg(&path)
        .arg("test-pwd")
        .write_stdin(format!("test123\n{new_password}\n{new_password}\n"))
        .assert()
        .success()
        .stdout("")
        .stderr("");
    let updated_db = Database::open(
        &mut File::open(&path).unwrap(),
        DatabaseKey::new().with_password("test123"),
    )
    .unwrap();
    let updated = password_entry(&updated_db);
    assert_eq!(updated.get_password(), Some(new_password));
    let history = updated.history.as_ref().unwrap().get_entries();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].fields, original.fields);
}

#[cfg(unix)]
#[test]
fn test_pwd_edit_preserves_symlink_and_permissions() {
    use std::{
        fs::Permissions,
        os::unix::fs::{self as unix_fs, PermissionsExt},
    };

    let directory = TempDir::new().unwrap();
    let target = directory.path().join("database.kdbx");
    let link = directory.path().join("linked.kdbx");
    create_editable_database(&target);
    fs::set_permissions(&target, Permissions::from_mode(0o600)).unwrap();
    unix_fs::symlink(&target, &link).unwrap();
    edit_password(&link, "test123\nnew-dummy-password\nnew-dummy-password\n")
        .success()
        .stdout("")
        .stderr("");
    assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        password_entry(&open_test_database(&target)).get_password(),
        Some("new-dummy-password")
    );
}

#[test]
fn test_pwd_edit_readonly_database_is_not_replaced() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    create_editable_database(&path);
    let before = fs::read(&path).unwrap();
    let original_permissions = fs::metadata(&path).unwrap().permissions();
    let mut readonly_permissions = original_permissions.clone();
    readonly_permissions.set_readonly(true);
    fs::set_permissions(&path, readonly_permissions).unwrap();
    edit_password(&path, "test123\nnew-dummy-password\nnew-dummy-password\n")
        .failure()
        .stdout("")
        .stderr("Database is read-only\n");
    assert_eq!(fs::read(&path).unwrap(), before);
    fs::set_permissions(&path, original_permissions).unwrap();
}

#[test]
fn test_pwd_edit_ambiguous_entry_does_not_modify_database() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    let mut db = open_test_database(Path::new("tests/files/test.kdbx"));
    db.config.version = DatabaseConfig::default().version;
    db.root_mut()
        .add_entry()
        .set_unprotected(fields::TITLE, "test-pwd");
    db.save(&mut File::create(&path).unwrap(), test_database_key())
        .unwrap();
    let before = fs::read(&path).unwrap();
    edit_password(&path, "test123\nnew-dummy-password\nnew-dummy-password\n")
        .failure()
        .stdout("")
        .stderr("Ambiguous entry `test-pwd` (2 matches)\n");
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn test_pwd_edit_unsupported_version_does_not_modify_database() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    fs::copy("tests/files/kdbx3_1.kdbx", &path).unwrap();
    let original_db = Database::open(
        &mut File::open(&path).unwrap(),
        DatabaseKey::new().with_password("demopass"),
    )
    .unwrap();
    assert_eq!(original_db.config.version, DatabaseVersion::KDB3(1));
    let original = all_entries(original_db.root()).into_iter().next().unwrap();
    let before = fs::read(&path).unwrap();
    cargo_bin_cmd!()
        .env_remove("KDBX_KEY_FILE")
        .args(["pwd", "--edit", "--no-interaction", "-d"])
        .arg(&path)
        .arg(original.get_title().unwrap())
        .write_stdin("demopass\nnew-dummy-password\nnew-dummy-password\n")
        .assert()
        .failure()
        .stdout("")
        .stderr("Unsupported database version\n");
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn test_pwd_edit_missing_entry_does_not_modify_database() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("database.kdbx");
    fs::copy("tests/files/test.kdbx", &path).unwrap();
    let before = fs::read(&path).unwrap();
    cargo_bin_cmd!()
        .args(["pwd", "--edit", "--no-interaction", "-d"])
        .arg(&path)
        .args(["-k", "tests/files/secret", "unknown-pwd"])
        .write_stdin("test123\nnew-dummy-password\nnew-dummy-password\n")
        .assert()
        .failure()
        .stdout("")
        .stderr("Not found\n");
    assert_eq!(fs::read(&path).unwrap(), before);
}
