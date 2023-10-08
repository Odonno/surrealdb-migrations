DEFINE TABLE unpublish_post SCHEMALESS;

DEFINE FIELD post_id ON unpublish_post;
DEFINE FIELD created_at ON unpublish_post TYPE datetime DEFAULT time::now();

DEFINE EVENT unpublish_post ON TABLE unpublish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "DRAFT" WHERE id = $after.post_id
);