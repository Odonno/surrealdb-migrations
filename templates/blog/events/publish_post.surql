DEFINE TABLE publish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD post_id ON publish_post TYPE record<post>;
DEFINE FIELD created_at ON publish_post TYPE datetime VALUE time::now() READONLY;

DEFINE EVENT publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "PUBLISHED" WHERE id = $after.post_id
);