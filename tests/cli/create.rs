use assert_cmd::Command;

#[test]
fn create_schema_file_dry_run() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    cmd.arg("create")
        .arg("schema")
        .arg("post")
        .arg("-f")
        .arg("name,title,published_at")
        .arg("--dry-run");

    cmd.assert().success();

    cmd.assert().stdout(
        "DEFINE TABLE post SCHEMALESS;

DEFINE FIELD name ON post;
DEFINE FIELD title ON post;
DEFINE FIELD published_at ON post;\n",
    );
}

#[test]
fn create_event_file_dry_run() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

    cmd.arg("create")
        .arg("event")
        .arg("publish_post")
        .arg("-f")
        .arg("post_id,created_at")
        .arg("--dry-run");

    cmd.assert().success();

    cmd.assert().stdout(
        "DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post;
DEFINE FIELD created_at ON publish_post;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    # TODO
);\n",
    );
}
