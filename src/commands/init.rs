use std::{fs::File, io::Read, path::PathBuf};

use keepass::{db::Database, DatabaseKey};

use crate::{pwd::Pwd, Result, STDIN};

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
    let password = if password.is_empty() {
        None
    } else {
        Some(&password[..])
    };

    let database_name = read_db_name();

    let mut db = Database::new(Default::default());

    db.meta.database_name = Some(database_name);

    let mut keyfile = args.key_file.and_then(|f| File::open(f).ok());
    let keyfile = keyfile.as_mut().map(|kf| kf as &mut dyn Read);
    let key = DatabaseKey { password, keyfile };

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
