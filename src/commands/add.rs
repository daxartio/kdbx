use std::path::PathBuf;

use keepass::db::{Entry, Node, Value};

use crate::{keepass::save_database, utils::open_database_interactively, Result, STDIN};

#[derive(clap::Args)]
pub struct Args {
    /// Store password for the database in the OS's keyring
    #[arg(short = 'p', long)]
    use_keyring: bool,

    /// Remove database's password from OS's keyring and exit
    #[arg(short = 'P', long)]
    remove_key: bool,

    /// KDBX file path
    #[arg(short, long, env = "KDBX_DATABASE")]
    database: PathBuf,

    /// Path to the key file unlocking the database
    #[arg(short, long, env = "KDBX_KEY_FILE")]
    key_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.database.exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, password) = open_database_interactively(
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
        put!("Username: ");
        STDIN.read_text()
    };
    let entry_password = {
        put!("Password: ");
        STDIN.read_password()
    };
    let totp_raw: crate::pwd::Pwd = {
        put!("TOTP (otpauth:// or secret): ");
        let totp_raw = STDIN.read_password();
        if totp_raw.starts_with("otpauth://") {
            totp_raw
        } else if !totp_raw.is_empty() {
            format!(
                "otpauth://totp/{}:{}?secret={}&period=30&digits=6&issuer={}",
                entry_title,
                entry_username,
                &totp_raw[..],
                entry_title
            )
            .into()
        } else {
            totp_raw
        }
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
    entry.fields.insert(
        "otp".to_string(),
        Value::Protected(totp_raw.as_bytes().into()),
    );
    let mut db = db?;
    db.root.children.push(Node::Entry(entry));

    save_database(db, &args.database, args.key_file.as_deref(), password);

    Ok(())
}
