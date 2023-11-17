use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn validates_check_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("-f").arg("check.yml");
    cmd.assert().success();

    Ok(())
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("-f").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}
