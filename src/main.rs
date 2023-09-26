#[macro_use]
mod utils;
mod clipboard;
mod commands;
mod keepass;
mod keyring;
mod pwd;
mod stdin;

use docopt::Docopt;
use once_cell::sync::Lazy;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde_derive::Deserialize;

use log::*;

use std::{env, error, fmt, path::PathBuf, process, result, sync::atomic, thread, time};

const DEFAULT_TIMEOUT: u8 = 15; // 15 seconds
const CANCEL_RQ_FREQ: u64 = 10; // ten times in a second

static BIN_NAME: &str = env!("CARGO_PKG_NAME");
static ENV_VAR_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "_DEFAULTS");
static USAGE: &str = "
BIN_NAME BIN_VERSION
    KeePass KDBX4 password reader.

Usage:
    BIN_NAME [options] [<command>] [<entry>]
    BIN_NAME --help

Commands:
    clip     Copy password and clear clipboard after specified amount of time.
             This is default command if no other provided. Alias `c`.

    info     Display entry's info. Alias `show`, `s`, `i`.

    add      Add new entry. Alias `a`.

    init     Init new database.

Options:
    -d, --database <file>       KDBX file path.
    -k, --key-file <keyfile>    Path to the key file unlocking the database.
    -p, --use-keyring           Store password for the database in the OS's keyring.
    -P, --remove-key            Remove database's password from OS's keyring and exit.
    -G, --no-group              Show entries without group(s).
    -v, --preview               Preview entry during picking.
    -f, --full-screen           Use all available screen for picker.
    -t, --timeout <seconds>     Timeout in seconds before clearing the clipboard.
                                Default to DEFAULT_TIMEOUT seconds. 0 means no clean-up.
    -h, --help
    -V, --version

Environment variables:
    ENV_VAR_NAME                 Set default arguments (see examples).

Examples:
    Open a database and copy password to the clipboard after selection:
      $ BIN_NAME --database /root/secrets.kdbx

    Set default database, secret file and options via environment variable:
      export ENV_VAR_NAME=\"-d$HOME/my.kdbx -k$HOME/.secret -pGt7\"

    Display selector and then print entry's info:
      $ BIN_NAME show

    Copy password if only single entry found otherwise display selector:
      $ BIN_NAME clip gmail

    `clip` command name can be omitted:
      $ BIN_NAME gmail

    Print password to STDOUT:
      $ BIN_NAME github.com | cat

    Read password from STDIN:
      $ cat /mnt/usb/key | BIN_NAME
";

static CANCEL: atomic::AtomicBool = atomic::AtomicBool::new(false);
static STDIN: Lazy<stdin::Stdin> = Lazy::new(stdin::Stdin::new);

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() {
    env_logger::init();

    set_ctrlc_handler();

    let args = get_args();

    if let Err(err) = match args.arg_command {
        Command::Clip => commands::clip::run(args),
        Command::Show => commands::show::run(args),
        Command::Init => commands::init::run(args),
        Command::Add => commands::add::run(args),
        Command::Unknown(cmd) => {
            Err(format!("Unknown command `{}`. Use `--help` to get more info.", cmd).into())
        }
    } {
        werr!("{}", err);
        process::exit(1);
    }
}

#[derive(Debug)]
enum Command {
    Clip,
    Show,
    Add,
    Init,
    Unknown(String),
}

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Command,
    arg_entry: Option<String>,
    flag_timeout: Option<u8>,
    flag_no_group: bool,
    flag_preview: bool,
    flag_full_screen: bool,
    flag_use_keyring: bool,
    flag_remove_key: bool,
    flag_database: Option<PathBuf>,
    flag_key_file: Option<PathBuf>,
}

impl Args {
    fn from_env(dopt: &Docopt) -> Args {
        let env_var = env::var(ENV_VAR_NAME.to_uppercase()).unwrap_or_default();

        let mut argv = "BIN_NAME clip ".to_string();
        argv.push_str(env_var.as_str().trim());

        let dopt = dopt.clone();
        dopt.argv(argv.split(' '))
            .deserialize()
            .unwrap_or_else(|_| {
                werr!("Invalid arguments in {}.", &ENV_VAR_NAME.to_uppercase());
                process::exit(1);
            })
    }

    fn from_cmdline(dopt: &Docopt) -> Args {
        let dopt = dopt.clone();
        dopt.help(true)
            .options_first(true)
            .version(Some(version()))
            .deserialize()
            .unwrap_or_else(|e| e.exit())
    }
}

fn get_args() -> Args {
    let usage = USAGE
        .replace("DEFAULT_TIMEOUT", &DEFAULT_TIMEOUT.to_string())
        .replace("ENV_VAR_NAME", &ENV_VAR_NAME.to_uppercase())
        .replace("BIN_NAME", BIN_NAME)
        .replace("BIN_VERSION", &version());

    let dopt = Docopt::new(usage).unwrap_or_else(|e| e.exit());

    let env = Args::from_env(&dopt);
    debug!("env var: {:#?}", env);

    let mut cmd = Args::from_cmdline(&dopt);
    debug!("cmd line: {:#?}", cmd);

    cmd.arg_command = match cmd.arg_command {
        Command::Unknown(ref command) if command.is_empty() => Command::Clip,

        Command::Unknown(ref entry) if cmd.arg_entry.is_none() => {
            cmd.arg_entry = Some(entry.to_string());
            Command::Clip
        }
        c => c,
    };

    cmd.flag_timeout = cmd
        .flag_timeout
        .or(env.flag_timeout)
        .or(Some(DEFAULT_TIMEOUT))
        .filter(|&t| t != 0);

    cmd.flag_use_keyring |= env.flag_use_keyring;
    cmd.flag_no_group |= env.flag_no_group;
    cmd.flag_preview |= env.flag_preview;
    cmd.flag_full_screen |= env.flag_full_screen;
    cmd.flag_key_file = cmd.flag_key_file.or(env.flag_key_file);
    cmd.flag_database = cmd.flag_database.or(env.flag_database).or_else(|| {
        werr!("No database file were found. Use `--help` to get more info.");
        process::exit(1);
    });

    if cmd.flag_remove_key {
        let dbfile = cmd.flag_database.as_deref().unwrap();

        if let Some(keyring) = keyring::Keyring::from_db_path(dbfile) {
            if let Err(msg) = keyring.delete_password() {
                werr!("No key removed for `{}`. {}", dbfile.to_string_lossy(), msg);
            }
        }

        process::exit(0);
    }

    debug!("merged: {:#?}", cmd);
    cmd
}

fn set_ctrlc_handler() {
    if let Err(e) = ctrlc::set_handler(|| {
        CANCEL.store(true, atomic::Ordering::SeqCst);
        STDIN.reset_tty();

        // allow gracefully finish any cancellable loop
        thread::sleep(time::Duration::from_millis(2 * 1_000 / CANCEL_RQ_FREQ));

        let _ = clipboard::set_clipboard(None);
        process::exit(1);
    }) {
        warn!("unable to setup Ctrl+C handler: {}", e);
    }
}

fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );
    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_owned(),
    }
}

struct CommandVisitor;

impl<'de> Visitor<'de> for CommandVisitor {
    type Value = Command;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`clip` or `show` commands")
    }

    fn visit_str<E>(self, s: &str) -> result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match &*s.to_lowercase() {
            "clip" | "c" => Command::Clip,
            "show" | "s" | "info" | "i" => Command::Show,
            "add" | "a" => Command::Add,
            "init" => Command::Init,
            cmd => Command::Unknown(cmd.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(d: D) -> result::Result<Command, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(CommandVisitor)
    }
}
