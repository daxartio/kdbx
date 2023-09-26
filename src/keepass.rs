use keepass::{error::DatabaseOpenError, Database, DatabaseKey};
use log::*;
use std::{
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
};

use crate::{keyring::Keyring, pwd::Pwd, utils::is_tty, STDIN};

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
    dbfile: &Path,
    keyfile: Option<&Path>,
    use_keyring: bool,
) -> (Result<Database, DatabaseOpenError>, Pwd) {
    let keyring = if use_keyring {
        Keyring::from_db_path(dbfile).map(|k| {
            debug!("keyring: {}", k);
            k
        })
    } else {
        None
    };

    if let Some(Ok(password)) = keyring.as_ref().map(|k| k.get_password()) {
        if let Ok(db) = open_db(password.clone(), dbfile, keyfile) {
            return (Ok(db), password);
        }

        warn!("removing wrong password in the keyring");
        let _ = keyring.as_ref().map(|k| k.delete_password());
    }

    // Try read password from pipe
    if !is_tty(io::stdin()) {
        let password = STDIN.read_password();
        return (open_db(password.clone(), dbfile, keyfile), password);
    }

    // Allow multiple attempts to enter the password from TTY
    let mut att: u8 = 3;
    loop {
        put!("Password: ");

        let password = STDIN.read_password();
        let db = open_db(password.clone(), dbfile, keyfile);

        // If opened successfully store the password
        if db.is_ok() {
            let _ = keyring.as_ref().map(|k| k.set_password(&password));
        }

        att -= 1;

        if db.is_ok() || att == 0 {
            break (db.map_err(From::from), password);
        }

        wout!("{} attempt(s) left.", att);
    }
}

fn open_db(
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
