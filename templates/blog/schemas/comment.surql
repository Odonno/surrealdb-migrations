# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS;

DEFINE FIELD content ON comment TYPE string ASSERT $value != NONE;
DEFINE FIELD created_at ON comment TYPE datetime VALUE $before OR time::now();