# Keepass CLI

[![Crates.io](https://img.shields.io/crates/v/kdbx.svg)](https://crates.io/crates/kdbx)

A secure hole for your passwords (Keepass CLI)

## Features

- MacOS, Linux
- master password is stored in OS's keyring

## Install

### Manual

[Download](https://github.com/daxartio/kdbx/releases)

### Cargo

```
cargo install kdbx
```

[cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```
cargo binstall kdbx
```

### Script

```
curl -fsSL https://raw.githubusercontent.com/daxartio/kdbx/master/install.sh | sh -s
```

### Homebrew

```
brew install daxartio/tap/kdbx
```

**Please take a backup of your database before updating the application.**

## Usage

### Examples

Display selector and then print entry's info:

```
kdbx show
```

Copy password/totp if only single entry found otherwise display selector:

```
kdbx pwd /root/emails/gmail
kdbx totp /root/emails/gmail
```

Print password/totp to STDOUT:

```
kdbx pwd github.com | cat
kdbx totp github.com | cat
```

Read password from STDIN:

```
cat /mnt/usb/key | kdbx pwd
```

Generate shell completions:

```
kdbx completion --shell=zsh > /usr/local/share/zsh/site-functions/_kdbx
```

```
kdbx completion --shell=fish > ~/.config/fish/completions/kdbx.fish
```

<!-- CLI START -->

### commands

```
A secure hole for your passwords (Keepass CLI)

Usage: kdbx <COMMAND>

Commands:
  pwd         Copy password and clear clipboard after specified amount of time
  totp        Copy totp
  show        Display entry's info
  add         Add new entry
  init        Init new database
  list        List all entries
  completion  Outputs the completion file for given shell
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### pwd

```
Copy password and clear clipboard after specified amount of time

Usage: kdbx pwd [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -t, --timeout <TIMEOUT>    Timeout in seconds before clearing the clipboard. 0 means no clean-up [default: 15]
  -G, --no-group             Show entries without group(s)
  -n, --no-interaction       Do not ask any interactive question
  -v, --preview              Preview entry during picking
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### totp

```
Copy totp

Usage: kdbx totp [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -G, --no-group             Show entries without group(s)
  -n, --no-interaction       Do not ask any interactive question
  -v, --preview              Preview entry during picking
      --raw                  Show the secret instead of code
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### show

```
Display entry's info

Usage: kdbx show [OPTIONS] --database <DATABASE> [ENTRY]

Arguments:
  [ENTRY]

Options:
  -G, --no-group             Show entries without group(s)
  -n, --no-interaction       Do not ask any interactive question
  -v, --preview              Preview entry during picking
  -f, --full-screen          Use all available screen for picker
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### add

```
Add new entry

Usage: kdbx add [OPTIONS] --database <DATABASE>

Options:
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### init

```
Init new database

Usage: kdbx init [OPTIONS] --database <DATABASE>

Options:
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### list

```
List all entries

Usage: kdbx list [OPTIONS] --database <DATABASE>

Options:
  -G, --no-group             Show entries without group(s)
  -p, --use-keyring          Store password for the database in the OS's keyring
  -P, --remove-key           Remove database's password from OS's keyring and exit
  -d, --database <DATABASE>  KDBX file path [env: KDBX_DATABASE=]
  -k, --key-file <KEY_FILE>  Path to the key file unlocking the database [env: KDBX_KEY_FILE=]
  -h, --help                 Print help
```

### completion

```
Outputs the completion file for given shell

Usage: kdbx completion --shell <SHELL>

Options:
  -s, --shell <SHELL>  [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help           Print help
```

<!-- CLI END -->

## Configuration

kdbx can be configured using environment variables.

| name           | type   | default | options             |
|----------------|--------|---------|---------------------|
| KDBX_LOG       | string | 'error' | ...                 |
| KDBX_LOG_STYLE | string | 'auto'  | auto, always, never |

## License

MIT
