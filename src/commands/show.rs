use keepass::db::NodeRef;

use crate::{
    keepass::open_database,
    utils::{get_entries, skim},
    Args, Result,
};

pub(crate) fn run(args: Args) -> Result<()> {
    if !args.flag_database.as_deref().unwrap().exists() {
        return Err("File does not exist".to_string().into());
    }
    let (db, _) = open_database(
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        args.flag_use_keyring,
    );
    let db = db?;
    let query = args.arg_entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let Some(NodeRef::Entry(entry)) = db.root.get(&[query]) {
            put!(
                "Title: {}\nUserName: {}\nUrl: {}\n",
                entry.get_title().unwrap_or_default(),
                entry.get_username().unwrap_or_default(),
                entry.get_url().unwrap_or_default(),
            );
            return Ok(());
        }
    }

    if let Some(entry) = skim(
        &get_entries(&db.root, "".to_string()),
        query,
        args.flag_no_group,
        args.flag_preview,
        args.flag_full_screen,
    ) {
        put!(
            "Title: {}\nUserName: {}\nUrl: {}\n",
            entry.get_title().unwrap_or_default(),
            entry.get_username().unwrap_or_default(),
            entry.get_url().unwrap_or_default(),
        );
        return Ok(());
    }

    Ok(())
}
