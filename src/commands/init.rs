use std::{fs::File, path::PathBuf};

use clap::ValueHint;
use keepass::db::Database;

use crate::{Result, STDIN, keepass::new_database_key, pwd::Pwd};

#[derive(clap::Args)]
pub struct Args {
    /// KDBX file path
    #[arg(short, long, env = "KDBX_DATABASE", value_hint = ValueHint::FilePath)]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE", value_hint = ValueHint::FilePath)]
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

    let key = new_database_key(args.key_file.as_deref(), password)?;

    db.save(&mut File::create(args.database)?, key)?;

    Ok(())
}

fn read_password(prompt: &str) -> Pwd {
    put!("{}", prompt);
    STDIN.read_password()
}

fn read_db_name() -> String {
    put!("Database name: ");
    STDIN.read_text()
}
