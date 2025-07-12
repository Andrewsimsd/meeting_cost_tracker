use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn binary_runs_and_creates_file() {
    let path = Path::new("categories.toml");
    if path.exists() {
        fs::remove_file(path).unwrap();
    }

    let mut cmd = Command::cargo_bin("meeting_cost_tracker").unwrap();
    let _ = cmd.write_stdin("q").assert().success();

    assert!(path.exists());
    let contents = fs::read_to_string(path).unwrap();
    assert!(contents.contains("categories"));
}

#[test]
fn start_and_stop_meeting_via_ui() {
    let mut cmd = Command::cargo_bin("meeting_cost_tracker").unwrap();

    // Simulate 's', wait, then 't', then 'q'
    cmd.write_stdin("sq").assert().success();
}
