use std::{io, os::unix::process::CommandExt, process::Command, time::Duration};

use assert_cmd::{
    Command as AssertCommand,
    cargo::{cargo_bin, cargo_bin_cmd},
};
use predicates::prelude::*;
use rstest::rstest;

#[test]
fn test_show() {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd
        .args([
            "show",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
            "test-pwd",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd
        .success()
        .stdout("Title: test-pwd\nUsername: test\nPassword: ******");
}

#[test]
fn test_show_no_interaction_not_found() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "show",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "unknown-pwd",
        "-n",
    ])
    .write_stdin("test123")
    .assert()
    .stderr("Not found\n");
}

#[rstest]
#[case(&[])]
#[case(&["--preview"])]
#[case(&["--full-screen"])]
#[case(&["--preview", "--full-screen", "--no-group"])]
fn test_show_picker_without_terminal(#[case] picker_args: &[&str]) {
    let mut process = Command::new(cargo_bin!());
    // SAFETY: setsid is async-signal-safe and this closure only detaches the child from the terminal.
    unsafe {
        process.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut cmd = AssertCommand::from_std(process);
    cmd.args([
        "show",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "unmatched-query",
    ])
    .args(picker_args)
    .timeout(Duration::from_secs(5))
    .write_stdin("test123")
    .assert()
    .success()
    .stdout("")
    .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn test_show_sensitive() {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd
        .args([
            "show",
            "--show-sensitive",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
            "test-pwd",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd
        .success()
        .stdout("Title: test-pwd\nUsername: test\nPassword: 1234");
}

#[test]
fn test_show_totp_masks_secret() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "show",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "test-totp",
    ])
    .write_stdin("test123")
    .assert()
    .success()
    .stdout(
        predicate::str::contains("otp: ******")
            .and(predicate::str::contains("JBSWY3DPEHPK3PXP").not())
            .and(predicate::str::contains("TOTP Code:").not()),
    );
}
