use std::{
    fs::File,
    time::{SystemTime, UNIX_EPOCH},
};

use assert_cmd::cargo::cargo_bin_cmd;
use keepass::{
    Database, DatabaseKey,
    db::{TOTP, fields},
};
use predicates::prelude::*;
use rstest::rstest;
use tempfile::TempDir;
use url::Url;

fn database_with_otp(otp: &str) -> TempDir {
    let directory = tempfile::tempdir().unwrap();
    let mut database = Database::new();
    database.root_mut().add_entry().edit(|entry| {
        entry.set_unprotected(fields::TITLE, "test-totp");
        entry.set_protected(fields::OTP, otp);
    });
    let mut file = File::create(directory.path().join("test.kdbx")).unwrap();
    database
        .save(
            &mut file,
            DatabaseKey::new().with_password("test-only-password"),
        )
        .unwrap();
    directory
}

#[rstest]
#[case("jbswy3dpehpk3pxp", "JBSWY3DPEHPK3PXP")]
#[case("JBSW Y3DP EHPK 3PXP", "JBSWY3DPEHPK3PXP")]
#[case("\tjb sw y3 dp eh pk 3p xp\r\n", "JBSWY3DPEHPK3PXP")]
#[case("JBSWY3DPEHPK3PXP", "JBSWY3DPEHPK3PXP")]
#[case("mzxw6===", "MZXW6===")]
#[case("MZXW6", "MZXW6===")]
fn test_totp_normalizes_secret(#[case] secret: &str, #[case] canonical_secret: &str) {
    let mut uri =
        Url::parse("otpauth://totp/test?period=45&digits=8&algorithm=SHA256&issuer=Test").unwrap();
    uri.query_pairs_mut().append_pair("secret", secret);
    let directory = database_with_otp(uri.as_str());
    let canonical: TOTP = format!(
        "otpauth://totp/test?period=45&digits=8&algorithm=SHA256&secret={canonical_secret}"
    )
    .parse()
    .unwrap();
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let output = cargo_bin_cmd!()
        .args(["totp", "-d"])
        .arg(directory.path().join("test.kdbx"))
        .arg("test-totp")
        .write_stdin("test-only-password")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let code = String::from_utf8(output).unwrap();
    assert!((before..=after).any(|timestamp| code == canonical.value_at(timestamp).code));

    cargo_bin_cmd!()
        .args(["totp", "--raw", "-d"])
        .arg(directory.path().join("test.kdbx"))
        .arg("test-totp")
        .write_stdin("test-only-password")
        .assert()
        .success()
        .stdout(uri.to_string());

    cargo_bin_cmd!()
        .args(["show", "--show-sensitive", "-d"])
        .arg(directory.path().join("test.kdbx"))
        .arg("test-totp")
        .write_stdin("test-only-password")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"TOTP Code: [0-9]{8}").unwrap());
}

#[test]
fn test_totp_rejects_invalid_secret_without_disclosing_it() {
    let directory = database_with_otp("otpauth://totp/test?secret=invalid-test-secret!");
    cargo_bin_cmd!()
        .args(["totp", "-d"])
        .arg(directory.path().join("test.kdbx"))
        .arg("test-totp")
        .write_stdin("test-only-password")
        .assert()
        .failure()
        .stdout("")
        .stderr("Base32 decoding error\n");
}

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
    .stderr("No OTP record found\n");
}
