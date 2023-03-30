use assert_cmd::Command;
use serial_test::serial;

use crate::helpers;

#[test]
#[serial]
fn scaffold_empty_template() {
    helpers::clear_files_dir();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    cmd.arg("scaffold").arg("empty");

    cmd.assert().success();

    assert_eq!(
        dir_diff::is_different("templates/empty", "tests-files").unwrap(),
        false
    );
}

#[test]
#[serial]
fn scaffold_blog_template() {
    helpers::clear_files_dir();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    cmd.arg("scaffold").arg("blog");

    cmd.assert().success();

    assert_eq!(
        dir_diff::is_different("templates/blog/schemas", "tests-files/schemas").unwrap(),
        false
    );

    assert_eq!(
        dir_diff::is_different("templates/blog/events", "tests-files/events").unwrap(),
        false
    );

    let migration_files = std::fs::read_dir("tests-files/migrations").unwrap();
    assert_eq!(migration_files.count(), 3);
}
