use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_list() {
    let mut cmd = cargo_bin_cmd!();
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
