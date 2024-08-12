DEFINE TABLE OVERWRITE publish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD OVERWRITE post_id ON publish_post TYPE record<post>;
DEFINE FIELD OVERWRITE created_at ON publish_post TYPE datetime VALUE time::now() READONLY;

DEFINE EVENT OVERWRITE publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "PUBLISHED" WHERE id = $after.post_id
);