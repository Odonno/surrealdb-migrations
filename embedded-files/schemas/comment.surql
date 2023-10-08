# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS;

DEFINE FIELD content ON comment TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime DEFAULT time::now();