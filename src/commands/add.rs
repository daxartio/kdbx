use std::path::PathBuf;

use clap::ValueHint;
use keepass::db::fields;

use crate::{Result, STDIN, keepass::save_database, utils::open_database_interactively};

#[derive(clap::Args)]
pub struct Args {
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
    let (mut db, password) = open_database_interactively(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
        false,
    )?;
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
        } else if !totp_raw.trim().is_empty() {
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

    db.root_mut().add_entry().edit(|entry| {
        entry.set_unprotected(fields::TITLE, entry_title);
        entry.set_unprotected(fields::USERNAME, entry_username);
        entry.set_protected(fields::PASSWORD, entry_password.as_ref());

        if !totp_raw.trim().is_empty() {
            entry.set_protected("otp", totp_raw.as_ref());
        }
    });

    save_database(db, &args.database, args.key_file.as_deref(), password)?;

    Ok(())
}
