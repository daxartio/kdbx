use assert_cmd::Command;

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn test_show() {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
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
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
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
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
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
