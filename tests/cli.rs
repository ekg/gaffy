use std::process::Command;

use assert_cmd::prelude::*;
//use predicates::prelude::*;

#[test]
fn gaffy_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("gaffy")?;
    cmd.args(&["--gfa", "tests/data/K-3138.gfa"]);
    cmd.arg("tests/data/K-3138.gaf");

    cmd.assert().success();

    Ok(())
}
