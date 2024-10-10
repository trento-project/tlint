use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn validates_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("-f").arg("tests/fixtures/check.yml");
    cmd.assert().success();

    Ok(())
}

#[test]
fn validates_incorrect_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint")
        .arg("-f")
        .arg("tests/fixtures/invalid_check.yml");
    cmd.assert().failure().stdout(predicate::str::contains(
        "  Parse error   - missing field `id` at line 2 column 1\n",
    ));

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
