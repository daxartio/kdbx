use std::path::PathBuf;

use clap::ValueHint;

use crate::{
    Result,
    keepass::{find_entry, get_entries, show_entry},
    utils::{open_database_interactively, skim},
};

#[derive(clap::Args)]
pub struct Args {
    entry: Option<String>,

    /// Show entries without group(s)
    #[arg(short = 'G', long)]
    no_group: bool,

    /// Do not ask any interactive question
    #[arg(short = 'n', long)]
    no_interaction: bool,

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
        args.no_interaction,
    )?;
    let query = args.entry.as_ref().map(String::as_ref);

    if let Some(query) = query
        && let Some(entry) = find_entry(query, &db.root)
    {
        put!("{}", show_entry(entry));
        return Ok(());
    }

    if args.no_interaction {
        return Err("Not found".to_string().into());
    }

    if let Some(wrapped_entry) = skim(
        &get_entries(&db.root, ""),
        query.map(String::from),
        args.no_group,
        args.preview,
        args.full_screen,
        false,
    ) {
        put!("{}", show_entry(wrapped_entry.entry));
        return Ok(());
    }

    Ok(())
}
