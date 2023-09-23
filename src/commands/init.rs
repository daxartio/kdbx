use crate::{pwd::Pwd, Args, Result, STDIN};
use keepass::{db::Database, DatabaseKey};
use std::{fs::File, io::Read};

pub(crate) fn run(args: Args) -> Result<()> {
    let password = read_password("Password: ");
    let confirm = read_password("Confirm: ");
    if password != confirm {
        return Err("Passwords do not match".into());
    }
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };

    let database_name = read_db_name();

    let mut db = Database::new(Default::default());

    db.meta.database_name = Some(database_name);

    let mut keyfile = args.flag_key_file.and_then(|f| File::open(f).ok());
    let keyfile = keyfile.as_mut().map(|kf| kf as &mut dyn Read);
    let key = DatabaseKey { password, keyfile };

    db.save(
        &mut File::create(args.flag_database.as_deref().unwrap())?,
        key,
    )?;

    Ok(())
}

fn read_password(promt: &str) -> Pwd {
    put!("{}", promt);
    STDIN.read_password()
}

fn read_db_name() -> String {
    put!("Database name: ");
    STDIN.read_text()
}
