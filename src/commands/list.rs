use std::path::PathBuf;

use clap::ValueHint;

use crate::{
    Result,
    keepass::{EntryPath, get_entries},
    utils::open_database_interactively,
};

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
    #[arg(short, long, env = "KDBX_DATABASE", value_hint = ValueHint::FilePath)]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE", value_hint = ValueHint::FilePath)]
    key_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.database.exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, _) = open_database_interactively(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
        false,
    )?;

    let entries = &get_entries(&db.root, "");
    for e in entries.iter() {
        if args.no_group {
            wout!("{}", e.get_title());
        } else {
            wout!("{}", e.entry_path());
        }
    }

    Ok(())
}
