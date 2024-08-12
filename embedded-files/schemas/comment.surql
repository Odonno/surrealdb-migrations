# in: user
# out: post, comment
DEFINE TABLE OVERWRITE comment SCHEMALESS;

DEFINE FIELD OVERWRITE content ON comment TYPE string;
DEFINE FIELD OVERWRITE created_at ON comment TYPE datetime DEFAULT time::now();