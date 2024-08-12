DEFINE TABLE OVERWRITE unpublish_post SCHEMALESS;

DEFINE FIELD OVERWRITE post_id ON unpublish_post;
DEFINE FIELD OVERWRITE created_at ON unpublish_post TYPE datetime DEFAULT time::now();

DEFINE EVENT OVERWRITE unpublish_post ON TABLE unpublish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "DRAFT" WHERE id = $after.post_id
);