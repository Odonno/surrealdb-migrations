DEFINE TABLE OVERWRITE publish_post SCHEMALESS;

DEFINE FIELD OVERWRITE post_id ON publish_post;
DEFINE FIELD OVERWRITE created_at ON publish_post TYPE datetime DEFAULT time::now();

DEFINE EVENT OVERWRITE publish_post ON TABLE publish_post WHEN $event == "CREATE" THEN (
    UPDATE post SET status = "PUBLISHED" WHERE id = $after.post_id
);