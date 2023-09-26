use keepass::db::{Entry, Node, Value};

use crate::{
    keepass::{open_database, save_database},
    Args, Result, STDIN,
};

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.flag_database.as_deref().unwrap().exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, password) = open_database(
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        args.flag_use_keyring,
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

    save_database(
        db,
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        password,
    );

    Ok(())
}
