use crate::keepass::open_database;
use crate::utils::get_entries;
use crate::Result;

use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Args {
    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Store password for the database in the OS's keyring
    #[arg(short = 'p', long)]
    use_keyring: bool,

    /// Remove database's password from OS's keyring and exit
    #[arg(short = 'P', long)]
    remove_key: bool,

    /// KDBX file path
    #[arg(short, long)]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long)]
    key_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.database.exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, _) = open_database(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
    );
    let db = db?;

    let entries = &get_entries(&db.root, "".to_string());
    for e in entries.iter() {
        if args.no_group {
            wout!("{}", e.entry.get_title().unwrap_or_default());
        } else {
            wout!("{}", e.entry_path());
        }
    }

    Ok(())
}
