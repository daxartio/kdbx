use assert_cmd::cargo::cargo_bin_cmd;
use rstest::*;

#[rstest]
#[case("-h")]
#[case("--help")]
#[case("help")]
fn test_help(#[case] arg: &str) {
    let mut cmd = cargo_bin_cmd!();
    let assert_cmd = cmd.arg(arg).assert();
    assert_cmd.success().stdout(predicates::str::starts_with(
        "A secure hole for your passwords (Keepass CLI)",
    ));
}
