DEFINE TABLE publish_post SCHEMALESS;

DEFINE FIELD post_id ON publish_post TYPE record(post);
DEFINE FIELD created_at ON publish_post TYPE datetime VALUE $before OR time::now();

DEFINE EVENT publish_post ON TABLE publish_post WHEN $before == NONE THEN (
    UPDATE post SET status = "PUBLISHED" WHERE id = $after.post_id
);