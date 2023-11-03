use std::{fs::File, path::PathBuf};

use keepass::db::Database;

use crate::{keepass::new_database_key, pwd::Pwd, Result, STDIN};

#[derive(clap::Args)]
pub struct Args {
    /// KDBX file path
    #[arg(short, long, env = "KDBX_DATABASE")]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE")]
    key_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    if args.database.exists() {
        return Err("File exists".to_string().into());
    }
    let password = read_password("Password: ");
    let confirm = read_password("Confirm: ");
    if password != confirm {
        return Err("Passwords do not match".into());
    }

    let database_name = read_db_name();

    let mut db = Database::new(Default::default());

    db.meta.database_name = Some(database_name);

    let key = new_database_key(args.key_file.as_deref(), password);

    db.save(&mut File::create(args.database)?, key)?;

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
