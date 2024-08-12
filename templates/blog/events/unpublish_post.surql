DEFINE TABLE OVERWRITE unpublish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD OVERWRITE post_id ON unpublish_post TYPE record<post>;
DEFINE FIELD OVERWRITE created_at ON unpublish_post TYPE datetime VALUE time::now() READONLY;

DEFINE EVENT OVERWRITE unpublish_post ON TABLE unpublish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "DRAFT" WHERE id = $after.post_id
);