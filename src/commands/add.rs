use std::path::PathBuf;

use keepass::db::{Entry, Node, Value};

use crate::{
    keepass::{open_database, save_database},
    Result, STDIN,
};

#[derive(clap::Args)]
pub struct Args {
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
    let (db, password) = open_database(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
    );
    let entry_title = {
        put!("Title: ");
        STDIN.read_text()
    };
    let entry_username = {
        put!("UserName: ");
        STDIN.read_text()
    };
    let entry_password = {
        put!("Password: ");
        STDIN.read_password()
    };

    let mut entry = Entry::new();
    entry
        .fields
        .insert("Title".to_string(), Value::Unprotected(entry_title));
    entry
        .fields
        .insert("UserName".to_string(), Value::Unprotected(entry_username));
    entry.fields.insert(
        "Password".to_string(),
        Value::Protected(entry_password.as_bytes().into()),
    );
    let mut db = db?;
    db.root.children.push(Node::Entry(entry));

    save_database(db, &args.database, args.key_file.as_deref(), password);

    Ok(())
}
