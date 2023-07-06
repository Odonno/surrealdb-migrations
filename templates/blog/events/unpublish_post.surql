DEFINE TABLE unpublish_post SCHEMALESS
    PERMISSIONS
        FOR select, create FULL
        FOR update, delete NONE;

DEFINE FIELD post_id ON unpublish_post TYPE record(post);
DEFINE FIELD created_at ON unpublish_post TYPE datetime VALUE $before OR time::now();

DEFINE EVENT unpublish_post ON TABLE unpublish_post WHEN $before == NONE THEN (
    UPDATE post SET status = "DRAFT" WHERE id = $after.post_id
);