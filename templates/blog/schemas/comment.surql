# in: user
# out: post, comment
DEFINE TABLE comment SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_comment IN $auth.permissions
        FOR update, delete WHERE in = $auth.id;

DEFINE FIELD content ON comment TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime DEFAULT time::now();