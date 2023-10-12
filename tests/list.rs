use assert_cmd::Command;

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn test_list() {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    let assert_cmd = cmd
        .args([
            "list",
            "-d",
            "tests/files/test.kdbx",
            "-k",
            "tests/files/secret",
        ])
        .write_stdin("test123")
        .assert();
    assert_cmd
        .success()
        .stdout(predicates::str::starts_with("/Root/test-pwd"));
}
