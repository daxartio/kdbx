use assert_cmd::Command;
use rstest::*;

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[rstest]
#[case("-h")]
#[case("--help")]
#[case("help")]
fn test_help(#[case] arg: &str) {
    let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
    let assert_cmd = cmd.arg(arg).assert();
    assert_cmd.success().stdout(predicates::str::starts_with(
        "A secure hole for your passwords (Keepass CLI)",
    ));
}
