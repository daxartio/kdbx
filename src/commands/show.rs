use std::path::PathBuf;

use keepass::db::NodeRef;

use crate::{
    keepass::open_database,
    utils::{get_entries, show_entry, skim},
    Result,
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Preview entry during picking
    #[arg(short = 'v', long)]
    preview: bool,

    /// Use all available screen for picker
    #[arg(short, long)]
    full_screen: bool,

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
    let query = args.entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let Some(NodeRef::Entry(entry)) = db.root.get(&[query]) {
            put!("{}", show_entry(entry));
            return Ok(());
        }
    }

    if let Some(entry) = skim(
        &get_entries(&db.root, "".to_string()),
        query,
        args.no_group,
        args.preview,
        args.full_screen,
        false,
    ) {
        put!("{}", show_entry(entry));
        return Ok(());
    }

    Ok(())
}
