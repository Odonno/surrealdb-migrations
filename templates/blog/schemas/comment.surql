DEFINE TABLE OVERWRITE comment SCHEMALESS
    TYPE RELATION IN user OUT post|comment
    PERMISSIONS
        FOR select FULL
        FOR create WHERE permission:create_comment IN $auth.permissions
        FOR update, delete WHERE in = $auth.id;

DEFINE FIELD OVERWRITE content ON comment TYPE string;
DEFINE FIELD OVERWRITE created_at ON comment TYPE datetime VALUE time::now() READONLY;