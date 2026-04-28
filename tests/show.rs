use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

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
