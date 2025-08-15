use assert_cmd::Command;
use predicates::prelude::*;

// Note: These tests verify the current behavior where main.rs just prints a message
// The actual CLI integration will be tested when task 10 (main program integration) is implemented

#[test]
fn test_program_runs_successfully() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("j2s - JSON to Schema Tool"));
}

#[test]
fn test_program_runs_with_args() {
    // Currently the program ignores arguments and just prints the message
    // This will change when the main program logic is implemented
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("j2s - JSON to Schema Tool"));
}

#[test]
fn test_program_runs_with_version() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("j2s - JSON to Schema Tool"));
}
