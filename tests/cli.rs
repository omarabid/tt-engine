use std::fs;

use assert_cmd::Command;

#[test]
fn test_input_file() {
    let mut cmd = Command::cargo_bin("transact").unwrap();

    // filename required
    cmd.assert().failure();

    // file exists
    cmd.arg("tests/data/input.csv").assert().success();

    let expected_output = fs::read_to_string("tests/data/output.csv").unwrap();
    cmd.arg("tests/data/input.csv")
        .assert()
        .stdout(expected_output);
}
