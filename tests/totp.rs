use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_totp() {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd
        .args([
            "totp",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
            "test-totp",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd
        .success()
        .stdout(predicate::function(|totp: &str| {
            totp.parse::<i32>().is_ok()
        }));
}

#[test]
fn test_totp_raw() {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd
        .args([
            "totp",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
            "test-totp",
            "--raw",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd.success().stdout(
        "otpauth://totp/test-totp:test?secret=JBSWY3DPEHPK3PXP&period=30&digits=6&issuer=test-totp",
    );
}

#[test]
fn test_totp_no_interaction_not_found() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "totp",
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
fn test_totp_missing_secret() {
    let mut cmd = cargo_bin_cmd!();
    cmd.args([
        "totp",
        "-d",
        "tests/files/test.kdbx",
        "-k",
        "tests/files/secret",
        "test-pwd",
    ])
    .write_stdin("test123")
    .assert()
    .failure()
    .stderr("Entry has no TOTP secret\n");
}
