use std::path::PathBuf;

use clap::ValueHint;
use keepass::db::fields;
use url::Url;

use crate::{
    Result, STDIN,
    keepass::save_database,
    utils::{DatabaseOpenResult, open_database_interactively},
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
    let DatabaseOpenResult::Opened(mut db, password) = open_database_interactively(
        &args.database,
        args.key_file.as_deref(),
        args.use_keyring,
        args.remove_key,
        false,
    )?
    else {
        return Ok(());
    };
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
            build_totp_uri(&entry_title, &entry_username, &totp_raw)?
        } else {
            totp_raw
        }
    };

    db.root_mut().add_entry().edit(|entry| {
        entry.set_unprotected(fields::TITLE, entry_title);
        entry.set_unprotected(fields::USERNAME, entry_username);
        entry.set_protected(fields::PASSWORD, entry_password.as_ref());

        if !totp_raw.trim().is_empty() {
            entry.set_protected(fields::OTP, totp_raw.as_ref());
        }
    });

    save_database(*db, &args.database, args.key_file.as_deref(), password)?;

    Ok(())
}

fn build_totp_uri(title: &str, username: &str, secret: &str) -> Result<crate::pwd::Pwd> {
    let mut url = Url::parse("otpauth://totp/")?;
    let label = format!("{title}:{username}");
    url.path_segments_mut()
        .map_err(|_| std::io::Error::other("invalid TOTP URL base"))?
        .clear()
        .push(&label);
    url.query_pairs_mut()
        .append_pair("secret", secret.trim())
        .append_pair("period", "30")
        .append_pair("digits", "6")
        .append_pair("issuer", title);

    Ok(url.to_string().into())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_build_totp_uri_encodes_input() {
        let uri = build_totp_uri("Acme & Co", "user@example.com", "ABCD EFG=").unwrap();
        let uri = uri.as_ref();

        assert!(!uri.contains(' '));
        assert!(uri.starts_with("otpauth://totp/"));

        let parsed = Url::parse(uri).unwrap();
        let query = parsed.query_pairs().into_owned().collect::<HashMap<_, _>>();

        assert_eq!(query.get("secret").unwrap(), "ABCD EFG=");
        assert_eq!(query.get("issuer").unwrap(), "Acme & Co");
        assert_eq!(query.get("period").unwrap(), "30");
        assert_eq!(query.get("digits").unwrap(), "6");
    }
}
