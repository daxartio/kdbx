use assert_cmd::Command;
use predicates::prelude::*;

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn test_totp() {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
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
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
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
