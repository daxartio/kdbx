# Keepass CLI

## Install

```
cargo install kdbx
```

## Usage

```
$ kdbx --help

kdbx 0.0.0
    KeePass KDBX4 password reader.

Usage:
    kdbx [options] [<command>] [<entry>]
    kdbx --help

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
                                Default to 15 seconds. 0 means no clean-up.
    -h, --help
    -V, --version

Environment variables:
    KDBX_DEFAULTS                 Set default arguments (see examples).

Examples:
    Open a database and copy password to the clipboard after selection:
      $ kdbx --database /root/secrets.kdbx

    Set default database, secret file and options via environment variable:
      export KDBX_DEFAULTS="-d$HOME/my.kdbx -k$HOME/.secret -pGt7"

    Display selector and then print entry's info:
      $ kdbx show

    Copy password if only single entry found otherwise display selector:
      $ kdbx clip gmail

    `clip` command name can be omitted:
      $ kdbx gmail

    Print password to STDOUT:
      $ kdbx github.com | cat

    Read password from STDIN:
      $ cat /mnt/usb/key | kdbx
```

**Please take a backup of your database before updating the application.**

## License

MIT
