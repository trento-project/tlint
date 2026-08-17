// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn validates_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("tests/fixtures/check.yml");
    cmd.assert().success();

    Ok(())
}

#[test]
fn validates_incorrect_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("tests/fixtures/invalid_check.yml");
    cmd.assert().failure().stdout(
        predicate::str::contains("Parse error (tests/fixtures/invalid_check.yml)")
            .and(predicate::str::contains(
                "missing field `id` at line 2 column 1\n",
            )),
    );

    let mut cmd = Command::cargo_bin("tlint")?;

    let expected_path = std::path::Path::new("tests/fixtures").join("invalid_check.yml");

    cmd.arg("lint").arg("tests/fixtures");
    cmd.assert().failure().stdout(
        predicate::str::contains("Parse error")
            .and(predicate::str::contains(expected_path.to_str().unwrap()))
            .and(predicate::str::contains("missing field `id`")),
    );

    Ok(())
}

#[test]
fn validates_incorrect_check_with_recoverable_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint")
        .arg("tests/fixtures/missing_field.yml");
    cmd.assert().failure().stdout(
        predicate::str::contains("Parse error 156F64 (tests/fixtures/missing_field.yml)")
            .and(predicate::str::contains("missing field `description`")),
    );

    Ok(())
}

#[test]
fn validates_deprecated_check() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    let expected_path = std::path::Path::new("tests/fixtures").join("deprecated_check.yml");

    cmd.arg("lint").arg("tests/fixtures");
    cmd.assert().failure().stdout(
        predicate::str::contains(expected_path.to_str().unwrap())
            .and(predicate::str::contains(
                "Property 'premium' is deprecated and will be removed in the future",
            )),
    );

    Ok(())
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tlint")?;

    cmd.arg("lint").arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Unable to open file 'test/file/doesnt/exist': No such file or directory",
    ));

    Ok(())
}
