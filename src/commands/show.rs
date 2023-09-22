use crate::{
    utils::{open_database, skim},
    Args, Result,
};

pub(crate) fn run(args: Args) -> Result<()> {
    let db = open_database(
        args.flag_database.as_deref().unwrap(),
        args.flag_key_file.as_deref(),
        args.flag_use_keyring,
    )?;

    let query = args.arg_entry.as_ref().map(String::as_ref);

    if let Some(query) = query {
        if let [entry] = db.find(query).as_slice() {
            wout!("-----");
            put!("{}", entry);
            wout!("-----");
            return Ok(());
        }
    }

    if let Some(entry) = skim(
        &db.entries(),
        query,
        args.flag_no_group,
        args.flag_preview,
        args.flag_full_screen,
    ) {
        wout!("-----");
        put!("{}", entry);
        wout!("-----");
        return Ok(());
    }

    Ok(())
}
