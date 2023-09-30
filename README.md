# Keepass CLI

[![Crates.io](https://img.shields.io/crates/v/kdbx.svg)](https://crates.io/crates/kdbx)

A secure hole for your passwords (Keepass CLI)

## Install

[Download](https://github.com/daxartio/kdbx/releases)

or

```
cargo install kdbx
```

## Usage

### commands

```
A secure hole for your passwords (Keepass CLI)

Usage: kdbx [COMMAND]

Commands:
  clip  Copy password and clear clipboard after specified amount of time
  totp  Copy totp
  show  Display entry's info
  add   Add new entry
  init  Init new database
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### clip

```
Copy password and clear clipboard after specified amount of time

Usage: kdbx clip [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -t, --timeout <TIMEOUT>    Timeout in seconds before clearing the clipboard. 0 means no clean-up [default: 15]
  -G, --no-group             Show entries without group(s)
  -v, --preview              Preview entry during picking
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database
  -h, --help                 Print help
  -V, --version              Print version
```

### totp

```
Copy totp

Usage: kdbx totp [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -G, --no-group             Show entries without group(s)
  -v, --preview              Preview entry during picking
      --raw                  Show the secret instead of code
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database
  -h, --help                 Print help
  -V, --version              Print version
```

### show

```
Display entry's info

Usage: kdbx show [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -G, --no-group             Show entries without group(s)
  -v, --preview              Preview entry during picking
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database
  -h, --help                 Print help
  -V, --version              Print version
```

### add

```
Add new entry

Usage: kdbx add [OPTIONS] --database <DATABASE>

Options:
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database
  -h, --help                 Print help
  -V, --version              Print version
```

### init

```
Init new database

Usage: kdbx init [OPTIONS] --database <DATABASE>

Options:
  -d, --database <DATABASE>  KDBX file path
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

```
Display selector and then print entry's info:
  $ kdbx show

Copy password if only single entry found otherwise display selector:
  $ kdbx clip gmail

Print password to STDOUT:
  $ kdbx clip github.com | cat

Read password from STDIN:
  $ cat /mnt/usb/key | kdbx clip
```

**Please take a backup of your database before updating the application.**

## License

MIT
